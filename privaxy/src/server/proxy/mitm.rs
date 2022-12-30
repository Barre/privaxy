use super::{exclusions::LocalExclusionStore, serve::serve};
use crate::{blocker::AdblockRequester, cert::CertCache, events::Event, statistics::Statistics};
use http::uri::{Authority, Scheme};
use hyper::{
    client::HttpConnector, http, server::conn::Http, service::service_fn, upgrade::Upgraded, Body,
    Method, Request, Response,
};
use hyper_rustls::HttpsConnector;
use std::{net::IpAddr, sync::Arc};
use tokio::{net::TcpStream, sync::broadcast};
use tokio_rustls::TlsAcceptor;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn serve_mitm_session(
    adblock_requester: AdblockRequester,
    hyper_client: hyper::Client<HttpsConnector<HttpConnector>>,
    client: reqwest::Client,
    req: Request<Body>,
    cert_cache: CertCache,
    broadcast_tx: broadcast::Sender<Event>,
    statistics: Statistics,
    client_ip_address: IpAddr,
    local_exclusion_store: LocalExclusionStore,
) -> Result<Response<Body>, hyper::Error> {
    let authority = match req.uri().authority().cloned() {
        Some(authority) => authority,
        None => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = http::StatusCode::BAD_REQUEST;

            log::warn!("Received a request without proper authority, sending bad request");

            return Ok(response);
        }
    };

    if Method::CONNECT == req.method() {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        let server_configuration =
            Arc::new(cert_cache.get(authority.clone()).await.server_configuration);

        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(mut upgraded) => {
                    let is_host_blacklisted = local_exclusion_store.contains(authority.host());

                    if is_host_blacklisted {
                        let _result = tunnel(&mut upgraded, &authority).await;

                        return;
                    }

                    let http = Http::new();

                    match TlsAcceptor::from(server_configuration)
                        .accept(upgraded)
                        .await
                    {
                        Ok(tls_stream) => {
                            let _result = http
                                .serve_connection(
                                    tls_stream,
                                    service_fn(move |req| {
                                        serve(
                                            adblock_requester.clone(),
                                            req,
                                            hyper_client.clone(),
                                            client.clone(),
                                            authority.clone(),
                                            Scheme::HTTPS,
                                            broadcast_tx.clone(),
                                            statistics.clone(),
                                            client_ip_address,
                                        )
                                    }),
                                )
                                .with_upgrades()
                                .await;
                        }
                        // Couldn't perform the tls handshake, they may only support TLS features that we don't or
                        // make use of untrusted certificates. Let's add them to a blacklist so we'll be able to
                        // tunnel them instead of trying to perform MITM.
                        // No blocking will be able to be performed.
                        Err(error) => {
                            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                                log::warn!("Unable to perform handshake for host: {}. Consider excluding it from blocking. The service may not tolerate TLS interception.", authority);
                            }
                        }
                    }
                }
                Err(e) => log::error!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(Body::empty()))
    } else {
        // The request is not of method `CONNECT`. Therefore,
        // this request is for an HTTP resource.
        serve(
            adblock_requester,
            req,
            hyper_client.clone(),
            client.clone(),
            authority,
            Scheme::HTTP,
            broadcast_tx,
            statistics,
            client_ip_address,
        )
        .await
    }
}

async fn tunnel(mut upgraded: &mut Upgraded, authority: &Authority) -> std::io::Result<()> {
    let mut server = TcpStream::connect(authority.to_string()).await?;

    tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    log::debug!("Started tunneling host: {}", authority);

    Ok(())
}

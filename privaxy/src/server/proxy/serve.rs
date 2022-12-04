use super::html_rewriter::Rewriter;
use crate::blocker::AdblockRequester;
use crate::statistics::Statistics;
use crate::web_gui::events::Event;
use adblock::blocker::BlockerResult;
use http::uri::{Authority, Scheme};
use http::{StatusCode, Uri};
use hyper::body::Bytes;
use hyper::client::HttpConnector;
use hyper::{http, Body, Request, Response};
use hyper_rustls::HttpsConnector;
use std::net::IpAddr;
use tokio::sync::broadcast;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn serve(
    adblock_requester: AdblockRequester,
    request: Request<Body>,
    hyper_client: hyper::Client<HttpsConnector<HttpConnector>>,
    client: reqwest::Client,
    authority: Authority,
    scheme: Scheme,
    broadcast_sender: broadcast::Sender<Event>,
    statistics: Statistics,
    client_ip_address: IpAddr,
) -> Result<Response<Body>, hyper::Error> {
    let scheme_string = scheme.to_string();

    let uri = match http::uri::Builder::new()
        .scheme(scheme)
        .authority(authority)
        .path_and_query(match request.uri().path_and_query() {
            Some(path_and_query) => path_and_query.as_str(),
            None => "/",
        })
        .build()
    {
        Ok(uri) => uri,
        Err(_err) => {
            return Ok(get_empty_response(http::StatusCode::BAD_REQUEST));
        }
    };

    if request.headers().contains_key(http::header::UPGRADE) {
        return Ok(perform_two_ends_upgrade(request, uri, hyper_client).await);
    }

    let (mut parts, body) = request.into_parts();
    parts.uri = uri.clone();

    let (sender, new_body) = Body::channel();

    let req = Request::from_parts(parts, body);

    log::debug!("{} {}", req.method(), req.uri());

    statistics.increment_top_clients(client_ip_address);

    let (is_request_blocked, blocker_result) = adblock_requester
        .is_network_url_blocked(
            uri.to_string(),
            match req.headers().get(http::header::REFERER) {
                Some(referer) => referer.to_str().unwrap().to_string(),
                // When no referer, we default to `uri` as we otherwise may get many false
                // positives due to the blocker thinking it's third party requests.
                None => uri.to_string(),
            },
        )
        .await;

    let _result = broadcast_sender.send(Event {
        now: chrono::Utc::now(),
        method: req.method().to_string(),
        url: req.uri().to_string(),
        is_request_blocked,
    });

    if is_request_blocked {
        statistics.increment_blocked_requests();
        statistics.increment_top_blocked_paths(format!(
            "{}://{}{}",
            scheme_string,
            uri.host().unwrap(),
            uri.path()
        ));

        log::debug!("Blocked request: {}", uri);

        return Ok(get_blocked_by_privaxy_response(blocker_result));
    }

    let mut new_response = Response::new(new_body);

    let mut request_headers = req.headers().clone();
    request_headers.remove(http::header::CONNECTION);
    request_headers.remove(http::header::HOST);

    let mut response = match client
        .request(req.method().clone(), req.uri().to_string())
        .headers(request_headers)
        .body(req.into_body())
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => return Ok(get_informative_error_response(&err.to_string())),
    };

    statistics.increment_proxied_requests();

    *new_response.headers_mut() = response.headers().clone();

    let (mut parts, new_new_body) = new_response.into_parts();
    parts.status = response.status();

    let new_response = Response::from_parts(parts, new_new_body);

    if let Some(content_type) = response.headers().get(http::header::CONTENT_TYPE) {
        if let Ok(value) = content_type.to_str() {
            if value.contains("text/html") {
                let (sender_rewriter, receiver_rewriter) = crossbeam_channel::unbounded::<Bytes>();

                let rewriter = Rewriter::new(
                    uri.to_string(),
                    adblock_requester,
                    receiver_rewriter,
                    sender,
                    statistics,
                );

                tokio::task::spawn_blocking(|| rewriter.rewrite());

                while let Ok(Some(chunk)) = response.chunk().await {
                    if let Err(_err) = sender_rewriter.send(chunk) {
                        break;
                    }
                }

                return Ok(new_response);
            }
        }

        tokio::spawn(write_proxied_body(response, sender));

        return Ok(new_response);
    }

    tokio::spawn(write_proxied_body(response, sender));

    Ok(new_response)
}

fn get_informative_error_response(reason: &str) -> Response<Body> {
    let mut response_body = String::from(include_str!("../../resources/head.html"));
    response_body +=
        &include_str!("../../resources/error.html").replace("#{request_error_reson}#", reason);

    let mut response = Response::new(Body::from(response_body));
    *response.status_mut() = http::StatusCode::BAD_GATEWAY;

    response
}

fn get_blocked_by_privaxy_response(blocker_result: BlockerResult) -> Response<Body> {
    // We don't redirect to network urls due to security concerns.
    if let Some(resource) = blocker_result.redirect {
        let response = Response::new(Body::from(resource));

        return response;
    }

    let filter_information = match blocker_result.filter {
        Some(filter) => filter,
        None => "No information".to_string(),
    };

    let mut response_body = String::from(include_str!("../../resources/head.html"));
    response_body += &include_str!("../../resources/blocked_by_privaxy.html")
        .replace("#{matching_filter}#", &filter_information);

    let mut response = Response::new(Body::from(response_body));
    *response.status_mut() = http::StatusCode::FORBIDDEN;

    response
}

fn get_empty_response(status_code: http::StatusCode) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = status_code;

    response
}

async fn write_proxied_body(mut response: reqwest::Response, mut sender: hyper::body::Sender) {
    while let Ok(Some(chunk)) = response.chunk().await {
        // The other end is broken, let's abort immediately.
        if let Err(_err) = sender.send_data(chunk).await {
            break;
        }
    }
}

/// When we receive a request to perform an upgrade, we need to initiate a bidirectional tunnel.
/// We upgrade the request towards the target server, towards the proxy end and we connect both through a duplex stream.
async fn perform_two_ends_upgrade(
    request: Request<Body>,
    uri: Uri,
    hyper_client: hyper::Client<HttpsConnector<HttpConnector>>,
) -> Response<Body> {
    let (mut duplex_client, mut duplex_server) = tokio::io::duplex(32);

    let mut new_request = Request::new(Body::empty());
    *new_request.headers_mut() = request.headers().clone();
    *new_request.uri_mut() = uri;

    tokio::spawn(async move {
        match hyper::upgrade::on(request).await {
            Ok(mut upgraded_client) => {
                let _result =
                    tokio::io::copy_bidirectional(&mut upgraded_client, &mut duplex_client).await;
            }
            Err(e) => {
                log::debug!("Unable to upgrade: {}", e)
            }
        }
    });

    let response = match hyper_client.request(new_request).await {
        Ok(response) => response,
        Err(_err) => return get_empty_response(http::StatusCode::BAD_REQUEST),
    };

    let mut new_response = get_empty_response(StatusCode::SWITCHING_PROTOCOLS);
    *new_response.headers_mut() = response.headers().clone();

    match hyper::upgrade::on(response).await {
        Ok(mut upgraded_server) => {
            tokio::spawn(async move {
                let _result =
                    tokio::io::copy_bidirectional(&mut upgraded_server, &mut duplex_server).await;
            });
        }
        Err(e) => {
            log::debug!("Unable to upgrade: {}", e)
        }
    }

    new_response
}

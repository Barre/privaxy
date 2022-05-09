use crate::blocker::AdblockRequester;
use crate::proxy::exclusions::LocalExclusionStore;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Server};
use include_dir::{include_dir, Dir};
use reqwest::redirect::Policy;
use std::collections::HashSet;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;

mod blocker;
mod blocker_utils;
mod ca;
mod cert;
mod configuration;
mod proxy;
mod statistics;
mod web_gui;

pub static WEBAPP_FRONTEND_DIR: Dir<'_> = include_dir!("web_frontend/dist");

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const RUST_LOG_ENV_KEY: &str = "RUST_LOG";

#[tokio::main]
async fn main() {
    let lib_rs = WEBAPP_FRONTEND_DIR.get_file("index.html").unwrap();
    println!("{:?}", String::from_utf8(lib_rs.contents().to_vec()));

    // We way need more logs to perform debugging or troubleshooting.
    // Let's only set default logging when "RUST_LOG" is not already set.
    if std::env::var(RUST_LOG_ENV_KEY).is_err() {
        std::env::set_var(RUST_LOG_ENV_KEY, "privaxy=info");
    }

    env_logger::init();

    // We use reqwest instead of hyper's client to perform most of the proxying as it's more convenient
    // to handle compression as well as offers a more convenient interface.
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .redirect(Policy::none())
        .trust_dns(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .unwrap();

    let configuration = match configuration::Configuration::read_from_home(client.clone()).await {
        Ok(configuration) => configuration,
        Err(err) => {
            println!(
                "An error occured while trying to process the configuration file: {:?}",
                err
            );
            std::process::exit(1)
        }
    };

    let local_exclusion_store = LocalExclusionStore::new(HashSet::from_iter(
        configuration.exclusions.clone().into_iter(),
    ));

    let ca_certificate = match configuration.ca_certificate() {
        Ok(ca_certificate) => ca_certificate,
        Err(err) => {
            println!("Unable to decode ca certificate: {:?}", err);
            std::process::exit(1)
        }
    };

    let ca_certificate_pem = std::str::from_utf8(&ca_certificate.to_pem().unwrap())
        .unwrap()
        .to_string();

    let ca_private_key = match configuration.ca_private_key() {
        Ok(ca_private_key) => ca_private_key,
        Err(err) => {
            println!("Unable to decode ca private key: {:?}", err);
            std::process::exit(1)
        }
    };

    let cert_cache = cert::CertCache::new(ca_certificate, ca_private_key);

    let statistics = statistics::Statistics::new();

    let (broadcast_tx, _broadcast_rx) = broadcast::channel(32);

    let blocking_disabled_store = Arc::new(std::sync::RwLock::new(false));

    let (crossbeam_sender, crossbeam_receiver) = crossbeam_channel::unbounded();
    let blocker_sender = crossbeam_sender.clone();

    let blocker_requester = AdblockRequester::new(blocker_sender);

    let configuration_updater = configuration::ConfigurationUpdater::new(
        configuration.clone(),
        client.clone(),
        blocker_requester.clone(),
        None,
    )
    .await;

    let configuration_updater_tx = configuration_updater.tx.clone();
    configuration_updater_tx.send(configuration).await.unwrap();

    configuration_updater.start();

    let configuration_save_lock = Arc::new(tokio::sync::Mutex::new(()));

    web_gui::start_web_gui_server(
        broadcast_tx.clone(),
        statistics.clone(),
        blocking_disabled_store.clone(),
        configuration_updater_tx.clone(),
        ca_certificate_pem,
        configuration_save_lock.clone(),
        local_exclusion_store.clone(),
    );

    thread::spawn(move || {
        let blocker = blocker::Blocker::new(
            crossbeam_sender,
            crossbeam_receiver,
            blocking_disabled_store,
        );

        blocker.handle_requests()
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 8100));

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    // The hyper client is only used to perform upgrades. We don't need to
    // handle compression.
    // Hyper's client don't follow redirects, which is what we want, nothing to
    // disable here.
    let hyper_client = Client::builder().build(https_connector);

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let client_ip_address = conn.remote_addr().ip();

        let client = client.clone();
        let hyper_client = hyper_client.clone();
        let cert_cache = cert_cache.clone();
        let blocker_requester = blocker_requester.clone();
        let broadcast_tx = broadcast_tx.clone();
        let statistics = statistics.clone();
        let local_exclusion_store = local_exclusion_store.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                proxy::serve_mitm_session(
                    blocker_requester.clone(),
                    hyper_client.clone(),
                    client.clone(),
                    req,
                    cert_cache.clone(),
                    broadcast_tx.clone(),
                    statistics.clone(),
                    client_ip_address,
                    local_exclusion_store.clone(),
                )
            }))
        }
    });

    let server = Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .tcp_keepalive(Some(Duration::from_secs(600)))
        .serve(make_service);

    log::info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        log::error!("server error: {}", e);
    }
}

use crate::proxy::exclusions::LocalExclusionStore;
use crate::statistics::Statistics;
use crate::WEBAPP_FRONTEND_DIR;
use crate::{blocker::BlockingDisabledStore, configuration::Configuration};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, mpsc::Sender};
use warp::http::Response;
use warp::path::Tail;
use warp::Filter;

pub(crate) mod blocking_enabled;
pub(crate) mod custom_filters;
pub(crate) mod events;
pub(crate) mod exclusions;
pub(crate) mod filters;
pub(crate) mod statistics;

#[derive(Debug, Serialize)]
pub(crate) struct ApiError {
    error: String,
}

pub(crate) fn start_web_gui_static_files_server(bind: SocketAddr, api_addr: SocketAddr) {
    let filter = warp::get().and(warp::path::tail()).map(move |tail: Tail| {
        let tail_str = tail.as_str();

        let mut is_index = tail_str == "index.html";

        let file_contents = match WEBAPP_FRONTEND_DIR.get_file(tail_str) {
            Some(file) => file.contents().to_vec(),
            None => {
                is_index = true;

                let index_html = WEBAPP_FRONTEND_DIR.get_file("index.html").unwrap();
                WEBAPP_FRONTEND_DIR.get_file("index.html").unwrap();

                index_html.contents().to_vec()
            }
        };

        let file_contents = if is_index {
            let index_utf8 = String::from_utf8(file_contents).unwrap();

            Vec::from(index_utf8.replace("{#api_host#}", &api_addr.to_string()))
        } else {
            file_contents
        };

        let mime = mime_guess::from_path(tail_str).first_raw().unwrap_or("");

        Response::builder()
            .header(http::header::CONTENT_TYPE, mime)
            .body(file_contents)
    });

    tokio::spawn(async move {
        warp::serve(filter).run(bind).await;
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn start_web_gui_server(
    events_sender: broadcast::Sender<events::Event>,
    statistics: Statistics,
    blocking_disabled_store: Arc<RwLock<bool>>,
    configuration_updater_sender: Sender<Configuration>,
    ca_certificate_pem: String,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
    local_exclusions_store: LocalExclusionStore,
    bind: SocketAddr,
) {
    let http_client = reqwest::Client::new();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "PUT"])
        .allow_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::CONTENT_LENGTH,
            http::header::DATE,
        ]);

    tokio::spawn(async move {
        let routes = warp::get()
            .and(
                warp::path("events")
                    .and(warp::ws())
                    .map(move |ws: warp::ws::Ws| {
                        let events_sender = events_sender.clone();

                        ws.on_upgrade(move |websocket| events::events(websocket, events_sender))
                    })
                    .or(warp::path("statistics")
                        .and(warp::ws())
                        .map(move |ws: warp::ws::Ws| {
                            let statistics = statistics.clone();

                            ws.on_upgrade(move |websocket| {
                                statistics::statistics(websocket, statistics)
                            })
                        })),
            )
            .or(warp::path("filters")
                .and(
                    warp::get()
                        .and(with_http_client(http_client.clone()))
                        .and_then(filters::get_filters_configuration),
                )
                .or(warp::put()
                    .and(warp::path("filters"))
                    .and(warp::body::json())
                    .and(with_http_client(http_client.clone()))
                    .and(with_configuration_updater_sender(
                        configuration_updater_sender.clone(),
                    ))
                    .and(with_configuration_save_lock(
                        configuration_save_lock.clone(),
                    ))
                    .and_then(filters::change_filter_status)))
            .or(warp::path("custom-filters")
                .and(
                    warp::get()
                        .and(with_http_client(http_client.clone()))
                        .and_then(custom_filters::get_custom_filters),
                )
                .or(warp::put().and(
                    warp::path("custom-filters")
                        .and(warp::body::json())
                        .and(with_http_client(http_client.clone()))
                        .and(with_configuration_updater_sender(
                            configuration_updater_sender.clone(),
                        ))
                        .and(with_configuration_save_lock(
                            configuration_save_lock.clone(),
                        ))
                        .and_then(custom_filters::put_custom_filters),
                )))
            .or(warp::path("exclusions")
                .and(
                    warp::get()
                        .and(with_http_client(http_client.clone()))
                        .and_then(exclusions::get_exclusions),
                )
                .or(warp::put().and(
                    warp::path("exclusions")
                        .and(warp::body::json())
                        .and(with_http_client(http_client.clone()))
                        .and(with_configuration_updater_sender(
                            configuration_updater_sender.clone(),
                        ))
                        .and(with_configuration_save_lock(configuration_save_lock))
                        .and(with_local_exclusions_store(local_exclusions_store))
                        .and_then(exclusions::put_exclusions),
                )))
            .or(warp::path("blocking-enabled")
                .and(
                    warp::get()
                        .and(with_blocking_disabled_store(
                            blocking_disabled_store.clone(),
                        ))
                        .and_then(blocking_enabled::get_blocking_enabled),
                )
                .or(warp::put()
                    .and(warp::path("blocking-enabled"))
                    .and(warp::body::json())
                    .and(with_blocking_disabled_store(blocking_disabled_store))
                    .and_then(blocking_enabled::put_blocking_enabled)))
            .or(
                warp::path("privaxy_ca_certificate.pem").and(warp::get().map(move || {
                    Response::builder()
                        .header(
                            http::header::CONTENT_DISPOSITION,
                            "attachment; filename=privaxy_ca_certificate.pem;",
                        )
                        .body(ca_certificate_pem.clone())
                })),
            )
            .or(warp::options().map(|| ""))
            .with(cors);

        warp::serve(routes).run(bind).await
    });
}

fn with_local_exclusions_store(
    local_exclusions_store: LocalExclusionStore,
) -> impl Filter<Extract = (LocalExclusionStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || local_exclusions_store.clone())
}

fn with_configuration_save_lock(
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> impl Filter<Extract = (Arc<tokio::sync::Mutex<()>>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || configuration_save_lock.clone())
}

fn with_blocking_disabled_store(
    blocking_disabled: BlockingDisabledStore,
) -> impl Filter<Extract = (BlockingDisabledStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || blocking_disabled.clone())
}

fn with_configuration_updater_sender(
    sender: Sender<Configuration>,
) -> impl Filter<Extract = (Sender<Configuration>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

fn with_http_client(
    http_client: reqwest::Client,
) -> impl Filter<Extract = (reqwest::Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || http_client.clone())
}

pub(crate) fn get_error_response(err: impl std::error::Error) -> Response<String> {
    Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(
            serde_json::to_string(&ApiError {
                error: format!("{:?}", err),
            })
            .unwrap(),
        )
        .unwrap()
}

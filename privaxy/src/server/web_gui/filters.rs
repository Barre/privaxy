use super::get_error_response;
use crate::configuration::Configuration;
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc::Sender;
use warp::http::Response;

#[derive(Debug, Deserialize)]
pub struct FilterStatusChangeRequest {
    enabled: bool,
    file_name: String,
}

pub async fn change_filter_status(
    filter_status_change_request: Vec<FilterStatusChangeRequest>,
    http_client: reqwest::Client,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<impl warp::Reply, Infallible> {
    let _guard = configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home(http_client).await {
        Ok(configuration) => configuration,
        Err(err) => {
            return Ok(get_error_response(err));
        }
    };

    for filter in filter_status_change_request {
        if let Err(err) = configuration
            .set_filter_enabled_status(&filter.file_name, filter.enabled)
            .await
        {
            return Ok(get_error_response(err));
        }
    }

    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(Response::builder()
        .status(http::StatusCode::ACCEPTED)
        .body("".to_string())
        .unwrap())
}

pub async fn get_filters_configuration(
    http_client: reqwest::Client,
) -> Result<impl warp::Reply, Infallible> {
    let configuration = match Configuration::read_from_home(http_client).await {
        Ok(configuration) => configuration,
        Err(err) => return Ok(get_error_response(err)),
    };

    let filters = configuration.filters;

    Ok(Response::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&filters).unwrap())
        .unwrap())
}

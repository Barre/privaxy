use super::get_error_response;
use crate::configuration::Configuration;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc::Sender;
use warp::http::StatusCode;

pub async fn get_custom_filters(
    http_client: reqwest::Client,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let configuration = match Configuration::read_from_home(http_client).await {
        Ok(configuration) => configuration,
        Err(err) => {
            return Ok(Box::new(get_error_response(err)));
        }
    };

    let custom_filters = configuration.custom_filters.join("\n");

    Ok(Box::new(warp::reply::json(&custom_filters)))
}

pub async fn put_custom_filters(
    custom_filters: String,
    http_client: reqwest::Client,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let _guard = configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home(http_client).await {
        Ok(configuration) => configuration,
        Err(err) => {
            return Ok(Box::new(get_error_response(err)));
        }
    };

    if let Err(err) = configuration.set_custom_filters(&custom_filters).await {
        return Ok(Box::new(get_error_response(err)));
    }

    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(Box::new(StatusCode::ACCEPTED))
}

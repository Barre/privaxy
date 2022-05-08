use crate::blocker::BlockingDisabledStore;
use serde::Deserialize;
use std::convert::Infallible;
use warp::http::StatusCode;

#[derive(Deserialize)]
pub struct BlockingEnabled(bool);

pub async fn get_blocking_enabled(
    blocking_disabled_store: BlockingDisabledStore,
) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(
        &!*blocking_disabled_store.read().unwrap(),
    ))
}

pub async fn put_blocking_enabled(
    blocking_enabled: BlockingEnabled,
    blocking_disabled_store: BlockingDisabledStore,
) -> Result<impl warp::Reply, Infallible> {
    *blocking_disabled_store.write().unwrap() = !blocking_enabled.0;

    Ok(StatusCode::NO_CONTENT)
}

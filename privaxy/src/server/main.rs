use privaxy::start_privaxy;
use std::time::Duration;

const RUST_LOG_ENV_KEY: &str = "RUST_LOG";

#[tokio::main]
async fn main() {
    if std::env::var(RUST_LOG_ENV_KEY).is_err() {
        std::env::set_var(RUST_LOG_ENV_KEY, "privaxy=info");
    }

    env_logger::init();

    start_privaxy().await;

    loop {
        tokio::time::sleep(Duration::from_secs(3600 * 24 * 30 * 365)).await
    }
}

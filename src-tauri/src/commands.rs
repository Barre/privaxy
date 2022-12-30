use privaxy::configuration::{Configuration, Filter};
use privaxy::{statistics::SerializableStatistics, PrivaxyServer};

use crate::FilterStatusChangeRequest;

#[tauri::command]
pub(crate) fn get_statistics(
    privaxy_server: tauri::State<'_, PrivaxyServer>,
) -> Result<SerializableStatistics, ()> {
    // https://github.com/tauri-apps/tauri/issues/2533
    Ok(privaxy_server.statistics.get_serialized())
}

#[tauri::command]
pub(crate) fn get_blocking_enabled(privaxy_server: tauri::State<'_, PrivaxyServer>) -> bool {
    !*privaxy_server.blocking_disabled_store.0.read().unwrap()
}

#[tauri::command]
pub(crate) fn set_blocking_enabled(enabled: bool, privaxy_server: tauri::State<'_, PrivaxyServer>) {
    *privaxy_server.blocking_disabled_store.0.write().unwrap() = !enabled;
}

#[tauri::command]
pub(crate) async fn get_custom_filters(
    http_client: tauri::State<'_, reqwest::Client>,
) -> Result<String, ()> {
    let configuration = match Configuration::read_from_home(http_client.inner().clone()).await {
        Ok(configuration) => configuration,
        Err(_) => return Err(()),
    };

    let custom_filters = configuration.custom_filters.join("\n");

    Ok(custom_filters)
}

#[tauri::command]
pub(crate) async fn set_custom_filters(
    input: String,
    privaxy_server: tauri::State<'_, PrivaxyServer>,
    http_client: tauri::State<'_, reqwest::Client>,
) -> Result<(), ()> {
    let _guard = privaxy_server.configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home(http_client.inner().clone()).await {
        Ok(configuration) => configuration,
        Err(_) => return Err(()),
    };

    if configuration.set_custom_filters(&input).await.is_err() {
        return Err(());
    }

    privaxy_server
        .configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(())
}

#[tauri::command]
pub(crate) async fn get_exclusions(
    http_client: tauri::State<'_, reqwest::Client>,
) -> Result<String, ()> {
    let configuration = match Configuration::read_from_home(http_client.inner().clone()).await {
        Ok(configuration) => configuration,
        Err(_) => return Err(()),
    };
    let exclusions = Vec::from_iter(configuration.exclusions.into_iter()).join("\n");

    Ok(exclusions)
}

#[tauri::command]
pub(crate) async fn set_exclusions(
    input: String,
    privaxy_server: tauri::State<'_, PrivaxyServer>,
    http_client: tauri::State<'_, reqwest::Client>,
) -> Result<(), ()> {
    let _guard = privaxy_server.configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home(http_client.inner().clone()).await {
        Ok(configuration) => configuration,
        Err(_) => return Err(()),
    };

    if configuration
        .set_exclusions(&input, privaxy_server.local_exclusion_store.clone())
        .await
        .is_err()
    {
        return Err(());
    }

    privaxy_server
        .configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(())
}

#[tauri::command]
pub(crate) async fn get_filters_configuration(
    http_client: tauri::State<'_, reqwest::Client>,
) -> Result<Vec<Filter>, ()> {
    let configuration = match Configuration::read_from_home(http_client.inner().clone()).await {
        Ok(configuration) => configuration,
        Err(_) => return Err(()),
    };

    Ok(configuration.filters)
}

#[tauri::command]
pub(crate) async fn change_filter_status(
    filter_status_change_request: Vec<FilterStatusChangeRequest>,
    privaxy_server: tauri::State<'_, PrivaxyServer>,
    http_client: tauri::State<'_, reqwest::Client>,
) -> Result<Vec<Filter>, ()> {
    let _guard = privaxy_server.configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home(http_client.inner().clone()).await {
        Ok(configuration) => configuration,
        Err(_) => return Err(()),
    };

    for filter in filter_status_change_request {
        if configuration
            .set_filter_enabled_status(&filter.file_name, filter.enabled)
            .await
            .is_err()
        {
            return Err(());
        }
    }

    privaxy_server
        .configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(configuration.filters)
}

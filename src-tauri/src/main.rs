#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use num_format::{Locale, ToFormattedString};
use privaxy::events::Event;
use privaxy::start_privaxy;
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, Window};
use tauri::{Manager, SystemTrayMenuItem};
use tokio::sync::broadcast;
use tokio::time;

mod commands;

const RUST_LOG_ENV_KEY: &str = "RUST_LOG";

#[derive(Debug, Deserialize)]
struct SaveCertificatePayload(PathBuf);

pub async fn stream_events(window: Window, events_sender: broadcast::Sender<Event>) {
    let mut events_receiver = events_sender.subscribe();

    while let Ok(event) = events_receiver.recv().await {
        window.emit("logged_request", event).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct FilterStatusChangeRequest {
    enabled: bool,
    file_name: String,
}

fn main() {
    if std::env::var(RUST_LOG_ENV_KEY).is_err() {
        std::env::set_var(RUST_LOG_ENV_KEY, "privaxy_app=info");
    }

    env_logger::init();

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = tokio_runtime.enter();

    let privaxy_server = tokio_runtime.block_on(async { start_privaxy().await });
    let privaxy_server_clone = privaxy_server.clone();
    let privaxy_ca_certificate = privaxy_server.ca_certificate_pem.clone();

    let mut proxied_requests = CustomMenuItem::new("Proxied requests", "Proxied requests: 0");
    proxied_requests.enabled = false;
    let mut blocked_requests = CustomMenuItem::new("Blocked requests", "Proxied requests: 0");
    blocked_requests.enabled = false;
    let mut modified_responses = CustomMenuItem::new("Modified responses", "Modified responses: 0");
    modified_responses.enabled = false;

    let open_app = CustomMenuItem::new("Open app".to_string(), "Open app");
    let quit = CustomMenuItem::new("Quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open_app)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(proxied_requests)
        .add_item(blocked_requests)
        .add_item(modified_responses)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let broadcast_sender = privaxy_server.requests_broadcast_sender.clone();

    tauri::Builder::default()
        .manage(privaxy_server)
        .manage(reqwest::Client::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_statistics,
            commands::get_blocking_enabled,
            commands::set_blocking_enabled,
            commands::get_custom_filters,
            commands::set_custom_filters,
            commands::get_exclusions,
            commands::set_exclusions,
            commands::get_filters_configuration,
            commands::change_filter_status
        ])
        .setup(move |app| {
            let main_window = app.get_window("main").unwrap();

            let app_handle = app.handle();

            tokio::spawn(async move {
                loop {
                    let statistics = privaxy_server_clone.statistics.get_serialized();

                    let proxied_requests = app_handle.tray_handle().get_item("Proxied requests");
                    let blocked_requests = app_handle.tray_handle().get_item("Blocked requests");
                    let modified_responses =
                        app_handle.tray_handle().get_item("Modified responses");

                    let _ = proxied_requests.set_title(format!(
                        "Proxied requests: {}",
                        statistics.proxied_requests.to_formatted_string(&Locale::en),
                    ));
                    let _ = blocked_requests.set_title(format!(
                        "Blocked requests: {}",
                        statistics.blocked_requests.to_formatted_string(&Locale::en),
                    ));
                    let _ = modified_responses.set_title(format!(
                        "Modified responses: {}",
                        statistics
                            .modified_responses
                            .to_formatted_string(&Locale::en),
                    ));

                    time::sleep(std::time::Duration::from_millis(400)).await;
                }
            });

            tokio::spawn(async { stream_events(main_window, broadcast_sender).await });

            let _ = app.listen_global("save_ca_file", move |event| {
                if let Some(path) = event.payload() {
                    let path = serde_json::from_str::<SaveCertificatePayload>(path)
                        .unwrap()
                        .0;

                    let mut file = match std::fs::File::create(path) {
                        Ok(file) => file,
                        Err(err) => {
                            log::error!("Unable to save ca file: {:?}", err);
                            return;
                        }
                    };
                    if let Err(err) = file.write_all(privaxy_ca_certificate.as_bytes()) {
                        log::error!("Unable to write ca file: {:?}", err)
                    }
                }
            });

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "Quit" => {
                    std::process::exit(0);
                }
                "Open app" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running privaxy");
}

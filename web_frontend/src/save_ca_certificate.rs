use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri_sys::dialog::FileDialogBuilder;
use tauri_sys::event::emit;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html};

#[derive(Serialize)]
struct SaveCertificatePayload(PathBuf);

pub struct SaveCaCertificate;

pub enum Message {
    SaveCaCertificate,
}

impl Component for SaveCaCertificate {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::SaveCaCertificate => {
                spawn_local(async move {
                    let path = FileDialogBuilder::new()
                        .add_filter("privaxy_ca_cert", &["pem"])
                        .set_default_path(&Path::new("privaxy_ca_cert.pem"))
                        .save()
                        .await
                        .unwrap();
                    if let Some(path) = path {
                        let _ = emit("save_ca_file", &SaveCertificatePayload(path)).await;
                    }
                });
            }
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let save_ca_certificate = ctx.link().callback(|_| Message::SaveCaCertificate);

        html! {
            <button onclick={save_ca_certificate}
                class="inline-flex items-center justify-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-white bg-gray-800 hover:bg-gray-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-100 focus:ring-gray-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="ml-0.5 mr-2 h-5 w-5" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                {"Save CA certificate"}
            </button>
        }
    }
}

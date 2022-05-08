use crate::blocking_enabled::BlockingEnabled;
use crate::get_api_host;
use futures::future::{AbortHandle, Abortable};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use num_format::{Locale, ToFormattedString};
use reqwasm::websocket::futures::WebSocket;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html};

#[derive(Deserialize)]
pub struct Message {
    proxied_requests: Option<u64>,
    blocked_requests: Option<u64>,
    modified_responses: Option<u64>,
    #[serde(with = "tuple_vec_map")]
    top_blocked_paths: Vec<(String, u64)>,
    #[serde(with = "tuple_vec_map")]
    top_clients: Vec<(String, u64)>,
}

pub struct Dashboard {
    message: Message,
    ws_abort_handle: AbortHandle,
}

impl Component for Dashboard {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let message_callback = ctx.link().callback(|message: Message| message);

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(
            async move {
                loop {
                    let ws = match WebSocket::open(&format!("ws://{}/statistics", get_api_host())) {
                        Ok(ws) => ws,
                        Err(_err) => {
                            log::warn!("Unable to connect to websocket, trying again.");

                            TimeoutFuture::new(1_000).await;

                            continue;
                        }
                    };

                    let (_write, mut read) = ws.split();

                    while let Some(Ok(msg)) = read.next().await {
                        let message = match msg {
                            reqwasm::websocket::Message::Text(s) => {
                                serde_json::from_str::<Message>(&s).unwrap()
                            }
                            reqwasm::websocket::Message::Bytes(_) => unreachable!(),
                        };

                        message_callback.emit(message);
                    }
                    log::warn!("Lost connection to websocket, trying again.");

                    TimeoutFuture::new(1_000).await;
                }
            },
            abort_registration,
        );

        spawn_local(async {
            let _result = future.await;
        });

        Self {
            ws_abort_handle: abort_handle,
            message: Message {
                proxied_requests: None,
                blocked_requests: None,
                modified_responses: None,
                top_blocked_paths: Vec::new(),
                top_clients: Vec::new(),
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.message = msg;

        // The server only sends new messages when there is actually
        // new data.
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        fn some_or_loading(s: Option<u64>) -> String {
            match s {
                Some(s) => s.to_formatted_string(&Locale::en),
                None => "Loading".to_string(),
            }
        }

        fn render_list_element(key: &str, count: u64) -> Html {
            html! {
            <li class="relative bg-white py-5 px-4">
                <div class="flex justify-between space-x-3">
                    <div class="min-w-0 flex-1">

                        <p class="text-sm font-medium text-gray-900 truncate">{ key }</p>
                    </div>
                    <div class="flex-shrink-0 whitespace-nowrap text-sm text-gray-500">{ count.to_formatted_string(&Locale::en) }</div>
                </div>
            </li>
                 }
        }

        html! {
            <>
                <div class="md:flex md:justify-between md:space-x-5">
                    <div class="pt-1.5">
                        <h1 class="text-2xl font-bold text-gray-900">{ "Dashboard" }<div
                                class=" mt-3 ml-3 inline pulsating-circle"></div>
                        </h1>
                    </div>
                    <div
                        class="mt-6 flex flex-col-reverse justify-stretch space-y-4 space-y-reverse sm:flex-row-reverse sm:justify-end sm:space-x-reverse sm:space-y-0 sm:space-x-3 md:mt-0 md:flex-row md:space-x-3">
                        <a href={format!("//{}/privaxy_ca_certificate.pem", get_api_host())}
                            class="inline-flex items-center justify-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-white bg-gray-800 hover:bg-gray-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-100 focus:ring-gray-500">
                            <svg xmlns="http://www.w3.org/2000/svg" class="ml-0.5 mr-2 h-5 w-5" fill="none"
                                viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>
                            {"Download CA certificate"}
                        </a>
                        <BlockingEnabled />
                    </div>
                </div>

                <dl
                    class="mt-5 grid grid-cols-1 rounded-lg bg-white overflow-hidden shadow divide-y divide-gray-200 md:grid-cols-3 md:divide-y-0 md:divide-x">
                    <div class="px-4 py-5 sm:p-6">
                        <dt class="text-base font-normal text-gray-900">
                            {"Proxied requests"}
                        </dt>
                        <dd class="mt-1 flex justify-between items-baseline md:block lg:flex">
                            <div class="flex items-baseline text-2xl font-semibold text-blue-600">
                                { some_or_loading(self.message.proxied_requests) }
                            </div>
                        </dd>
                    </div>

                    <div class="px-4 py-5 sm:p-6">
                        <dt class="text-base font-normal text-gray-900">
                            {"Blocked requests"}
                        </dt>
                        <dd class="mt-1 flex justify-between items-baseline md:block lg:flex">
                            <div class="flex items-baseline text-2xl font-semibold text-blue-600">
                                { some_or_loading(self.message.blocked_requests) }
                            </div>
                        </dd>
                    </div>

                    <div class="px-4 py-5 sm:p-6">
                        <dt class="text-base font-normal text-gray-900">
                            {"Modified responses"}
                        </dt>
                        <dd class="mt-1 flex justify-between items-baseline md:block lg:flex">
                            <div class="flex items-baseline text-2xl font-semibold text-blue-600">
                                { some_or_loading(self.message.modified_responses) }
                            </div>
                        </dd>
                    </div>
                </dl>
                <div class="mt-4 lg:grid lg:gap-y-4 lg:gap-x-8 lg:grid-cols-2">
                    <div class="mt-4 bg-white overflow-hidden shadow rounded-lg divide-y divide-gray-200">
                        <div class="px-4 py-5 sm:px-6">
                            <h3 class="text-lg font-medium">{"Top blocked paths"}</h3>
                        </div>
                        <div class="px-4 py-5 sm:p-6">
                            <ul role="list" class="divide-y divide-gray-200">
                                { for self.message.top_blocked_paths.iter().map(|(path,
                                count)|render_list_element(path, *count)) }
                            </ul>

                        </div>
                    </div>
                    <div class="mt-4 bg-white overflow-hidden shadow rounded-lg divide-y divide-gray-200">
                        <div class="px-4 py-5 sm:px-6">
                            <h3 class="text-lg font-medium">{"Top clients"}</h3>
                        </div>
                        <div class="px-4 py-5 sm:p-6">
                            <ul role="list" class="divide-y divide-gray-200">
                                { for self.message.top_clients.iter().map(|(client,
                                count)|render_list_element(client, *count)) }
                            </ul>
                        </div>
                    </div>
                </div>
            </>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.ws_abort_handle.abort()
    }
}

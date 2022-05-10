use crate::get_api_host;
use futures::future::{AbortHandle, Abortable};
use futures::StreamExt;
use reqwasm::websocket::futures::WebSocket;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html};

const MAX_REQUESTS_SHOWN: usize = 500;

#[derive(Deserialize)]
pub struct Message {
    now: String,
    method: String,
    url: String,
    is_request_blocked: bool,
}

pub struct Requests {
    messages: Vec<Message>,
    ws_abort_handle: AbortHandle,
}

impl Component for Requests {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let message_callback = ctx.link().callback(|message: Message| message);

        let ws = WebSocket::open(&format!("ws://{}/events", get_api_host())).unwrap();
        let (_write, mut read) = ws.split();

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(
            async move {
                while let Some(Ok(msg)) = read.next().await {
                    let message = match msg {
                        reqwasm::websocket::Message::Text(s) => {
                            serde_json::from_str::<Message>(&s).unwrap()
                        }
                        reqwasm::websocket::Message::Bytes(_) => unreachable!(),
                    };

                    message_callback.emit(message);
                }
            },
            abort_registration,
        );

        spawn_local(async {
            let _result = future.await;
        });

        Self {
            ws_abort_handle: abort_handle,
            messages: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.messages.insert(0, msg);

        self.messages.truncate(MAX_REQUESTS_SHOWN);

        // The server only sends new messages when there is actually
        // new data.
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        fn render_element(element: &Message) -> Html {
            let background = {
                if element.is_request_blocked {
                    "bg-red-50"
                } else {
                    ""
                }
            };

            html! {

            <tr class={ background }>
                <td class="w-1/12 px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {&element.now}
                </td>
                <td class="w-1/12 px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    <span
                        class="inline-flex items-center px-2.5 py-0.5 rounded-md text-sm font-medium bg-blue-100 text-blue-800">
                        {&element.method}
                    </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {&element.url}
                </td>
            </tr>
                }
        }

        html! {
               <>
          <h3 class="text-2xl font-bold text-gray-900 pt-1.5">
            {"Requests feed"}
            <div class="mt-2 ml-3 inline pulsating-circle"></div>
          </h3>
          <div class="mt-4 flex flex-col">
            <div class="-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
              <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
                <div class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
                  <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                      <tr>
                        <th scope="col"
                          class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          {"Timestamp"}
                        </th>
                        <th scope="col"
                          class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          {"Method"}
                        </th>
                        <th scope="col"
                          class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          {"Path"}
                        </th>
                      </tr>
                    </thead>
                    <tbody class="w-full bg-white divide-y divide-gray-200">
                      { for self.messages.iter().map(render_element) }
                    </tbody>
                  </table>
                </div>
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

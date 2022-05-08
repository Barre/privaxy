use crate::save_button;
use crate::submit_banner;
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::virtual_dom::VNode;
use yew::Properties;
use yew::{html, Component, Context, Html, InputEvent, TargetCast};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub h1: String,
    pub description: VNode,
    pub input_name: String,
    pub textarea_description: String,
    pub resource_url: String,
}

pub struct SettingsTextarea {
    is_save_button_enabled: bool,
    changes_saved: bool,
    input_data: String,
    previous_input_data: String,
}

pub enum Message {
    LoadCurrentState,
    UpdateInput(String),
    UpdatePreviousInputData,
    Save,
    Saved,
}

impl Component for SettingsTextarea {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Message::LoadCurrentState);

        Self {
            is_save_button_enabled: false,
            input_data: String::new(),
            previous_input_data: String::new(),
            changes_saved: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateInput(input_value) => {
                self.changes_saved = false;
                self.is_save_button_enabled = true;

                self.input_data = input_value;
            }
            Message::Save => {
                if !self.is_save_button_enabled {
                    return false;
                }

                let request = Request::put(&ctx.props().resource_url)
                    .header("Content-Type", "application/json")
                    .body(&serde_json::to_string(&self.input_data).unwrap());

                spawn_local(async move {
                    if let Ok(response) = request.send().await {
                        // Todo: Handle errors
                        if response.ok() {}
                    }
                });

                ctx.link().send_message(Message::Saved);
            }
            Message::Saved => {
                ctx.link().send_message(Message::UpdatePreviousInputData);

                self.changes_saved = true;
                self.is_save_button_enabled = false;
            }
            Message::LoadCurrentState => {
                let request = Request::get(&ctx.props().resource_url);

                let message_callback = ctx.link().callback(|message: Message| message);

                spawn_local(async move {
                    if let Ok(response) = request.send().await {
                        // Todo: Handle errors
                        if response.ok() {
                            if let Ok(response_content) = response.json::<String>().await {
                                message_callback.emit(Message::UpdateInput(response_content));
                                message_callback.emit(Message::UpdatePreviousInputData)
                            };
                        }
                    }
                });
            }
            Message::UpdatePreviousInputData => {
                self.previous_input_data = self.input_data.clone();
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        ctx.link().send_message(Message::UpdateInput(String::new()));
        ctx.link().send_message(Message::LoadCurrentState);

        self.changes_saved = false;

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let button_state =
            if !self.is_save_button_enabled || (self.input_data == self.previous_input_data) {
                save_button::SaveButtonState::Disabled
            } else {
                save_button::SaveButtonState::Enabled
            };

        let success_banner = if self.changes_saved {
            let icon = html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-white" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
            };
            html! {
                <submit_banner::SubmitBanner message="Changes saved" {icon} color={submit_banner::Color::Green}/>
            }
        } else {
            html! {}
        };

        let oninput = ctx.link().callback(|e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            let value = input.value();

            Message::UpdateInput(value)
        });

        let onclick = ctx.link().callback(|_| Message::Save);

        let props = ctx.props();

        html! {
            <>
            <div class="pt-1.5 mb-4">
                <h1 class="text-2xl font-bold text-gray-900">{ &props.h1 }</h1>
            </div>
            {props.description.clone()}

            {success_banner}

            <div class="mt-4">
                <label for={props.input_name.clone()} class="block text-sm font-medium text-gray-700">{&props.textarea_description}</label>
                <div class="mt-1">
                    <textarea {oninput} value={self.input_data.clone()} rows="8" name={props.input_name.clone()} id={props.input_name.clone()} class="shadow-sm focus:ring-blue-500 focus:border-blue-500 block w-full sm:text-sm border-gray-300 rounded-md"></textarea>
                </div>
            </div>
            <save_button::SaveButton state={button_state} {onclick} />
            </>
        }
    }
}

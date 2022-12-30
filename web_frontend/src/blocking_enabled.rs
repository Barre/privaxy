use serde::{Deserialize, Serialize};
use tauri_sys::tauri;
use wasm_bindgen_futures::spawn_local;
use yew::{classes, html, Component, Context, Html};

#[derive(Deserialize, Serialize)]
struct TauriBlockingEnabledArg {
    enabled: bool,
}

pub struct BlockingEnabled {
    blocking_enabled: bool,
}

#[derive(Debug)]
pub enum Message {
    EnableBlocking,
    DisableBlocking,
    BlockingEnabled,
    BlockingDisabled,
    SetCurrentBlockingState,
}

impl Component for BlockingEnabled {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Message::SetCurrentBlockingState);

        Self {
            blocking_enabled: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let message_callback = ctx.link().callback(|message: Message| message);
        log::error!("{:?}", &msg);
        match msg {
            Message::EnableBlocking => {
                spawn_local(async move {
                    match tauri::invoke::<_, ()>(
                        "set_blocking_enabled",
                        &TauriBlockingEnabledArg { enabled: true },
                    )
                    .await
                    {
                        Ok(_) => {
                            message_callback.emit(Message::BlockingEnabled);
                        }
                        Err(err) => {
                            log::error!("{:?}", err);
                            message_callback.emit(Message::BlockingDisabled)
                        }
                    }
                });
            }
            Message::DisableBlocking => {
                spawn_local(async move {
                    match tauri::invoke::<_, ()>(
                        "set_blocking_enabled",
                        &TauriBlockingEnabledArg { enabled: false },
                    )
                    .await
                    {
                        Ok(_) => {
                            message_callback.emit(Message::BlockingDisabled);
                        }
                        Err(err) => {
                            log::error!("{:?}", err);
                            message_callback.emit(Message::BlockingEnabled)
                        }
                    }
                });
            }
            Message::BlockingEnabled => {
                self.blocking_enabled = true;
            }
            Message::BlockingDisabled => {
                self.blocking_enabled = false;
            }
            Message::SetCurrentBlockingState => {
                spawn_local(async move {
                    match tauri::invoke::<_, bool>("get_blocking_enabled", &()).await {
                        Ok(bool_) => {
                            if bool_ {
                                message_callback.emit(Message::BlockingEnabled);
                            } else {
                                message_callback.emit(Message::BlockingDisabled);
                            }
                        }
                        Err(err) => {
                            log::error!("{:?}", err);
                        }
                    }
                });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let enable_blocking = ctx.link().callback(|_| Message::EnableBlocking);
        let disable_blocking = ctx.link().callback(|_| Message::DisableBlocking);

        let mut button_classes = classes!(
            "inline-flex",
            "items-center",
            "justify-center",
            "px-4",
            "py-2",
            "border",
            "transition",
            "ease-in-out",
            "duration-150",
            "border-transparent",
            "text-sm",
            "text-sm",
            "font-medium",
            "rounded-md",
            "shadow-sm",
            "text-white",
            "focus:outline-none",
            "focus:ring-2",
            "focus:ring-offset-2",
            "focus:ring-offset-gray-100",
        );

        if self.blocking_enabled {
            html! {
            <button onclick={disable_blocking} type="button"
                class={classes!(button_classes, "focus:ring-red-500", "bg-red-600", "hover:bg-red-700")}>
                <svg xmlns="http://www.w3.org/2000/svg" class="-ml-0.5 mr-2 h-5 w-5" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {"Pause blocking"}
            </button>
            }
        } else {
            html! {
            <button onclick={enable_blocking} type="button"
                class={classes!(button_classes, "focus:ring-green-500", "bg-green-600", "hover:bg-green-700")}>
                <svg xmlns="http://www.w3.org/2000/svg" class="-ml-0.5 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {"Resume blocking"}
            </button>
            }
        }
    }
}

use web_sys::MouseEvent;
use yew::{classes, html, Callback, Component, Context, Html, Properties};

#[derive(PartialEq, Eq)]
pub enum SaveButtonState {
    Loading,
    Enabled,
    Disabled,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: SaveButtonState,
    pub onclick: Callback<MouseEvent>,
}

pub struct SaveButton;

impl Component for SaveButton {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut save_button_classes = classes!(
            "inline-flex",
            "items-center",
            "justify-center",
            "focus:ring-blue-500",
            "bg-blue-600",
            "hover:bg-blue-700",
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

        let properties = ctx.props();

        if properties.state == SaveButtonState::Disabled
            || properties.state == SaveButtonState::Loading
        {
            save_button_classes.push("opacity-50");
            save_button_classes.push("cursor-not-allowed");
        }

        let button_text = if properties.state == SaveButtonState::Loading {
            "Loading..."
        } else {
            "Save changes"
        };

        html! {
            <button onclick={properties.onclick.clone()} type="button" class={classes!(save_button_classes, "mt-5" )}>
                <svg xmlns="http://www.w3.org/2000/svg" class="-ml-0.5 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                </svg>
                    {button_text}
            </button>
        }
    }
}

use yew::{classes, html, virtual_dom::VNode, Component, Context, Html, Properties};

pub struct SubmitBanner;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Green,
    Red,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub message: String,
    pub color: Color,
    pub icon: VNode,
}

impl Component for SubmitBanner {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let first_color = match props.color {
            Color::Green => "bg-green-500",
            Color::Red => "bg-red-500",
        };

        let second_color = match props.color {
            Color::Green => "bg-green-700",
            Color::Red => "bg-red-700",
        };

        html! {
            <div class={classes!("mb-5", "p-2", "rounded-lg", "shadow-lg", "sm:p-3", first_color)}>
                <div class="flex items-center justify-between flex-wrap">
                    <div class="w-0 flex-1 flex items-center">
                        <span class={classes!("flex", "p-2", "rounded-lg", second_color)}>
                            {props.icon.clone()}
                        </span>
                        <p class="ml-3 font-medium text-white truncate">
                            {&props.message}
                        </p>
                    </div>
                </div>
            </div>
        }
    }
}

use yew::{html, Component, Context, Html};

pub struct LoaderComponent;

impl Component for LoaderComponent {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="flex justify-center">
                <span class="loader ease-linear rounded-full border-4 border-t-4 border-white h-14 w-14"></span>
            </div>
        }
    }
}

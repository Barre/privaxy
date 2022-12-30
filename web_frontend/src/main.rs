use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod blocking_enabled;
mod dashboard;
mod filters;
mod requests;
mod save_button;
mod save_ca_certificate;
mod settings;
mod settings_textarea;
mod submit_banner;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Dashboard,
    #[at("/requests")]
    Requests,
    #[at("/settings/:s")]
    Settings,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(NotFound)]
fn not_found() -> Html {
    html! {
        <div class="bg-white min-h-full px-4 py-16 sm:px-6 sm:py-24 md:grid md:place-items-center lg:px-8">
        <div class="max-w-max mx-auto">
            <main class="sm:flex">
                <p class="text-4xl font-extrabold text-blue-600 sm:text-5xl">{"404"}</p>
                <div class="sm:ml-6">
                    <div class="sm:border-l sm:border-gray-200 sm:pl-6">
                        <h1 class="text-4xl font-extrabold text-gray-900 tracking-tight sm:text-5xl">{"Page not
                            found"}</h1>
                        <p class="mt-1 text-base text-gray-500">{"Please check the URL in the address bar and try
                            again."}</p>
                    </div>
                </div>
            </main>
        </div>
    </div>
     }
}

fn switch(route: &Route) -> Html {
    fn get_classes(current_route: Route, for_route_link: Route) -> Classes {
        if current_route == for_route_link {
            classes!(
                "bg-gray-900",
                "text-white",
                "px-3",
                "py-2",
                "rounded-md",
                "text-sm",
                "font-medium"
            )
        } else {
            classes!(
                "text-gray-300",
                "hover:bg-gray-700",
                "hover:text-white",
                "px-3",
                "py-2",
                "rounded-md",
                "text-sm",
                "font-medium"
            )
        }
    }

    let navigation = html! {
        <nav class="bg-gray-800">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <img class="h-8 w-auto text-white" src="/logo.svg" alt="Logo" />
            </div>
              <div class="flex ml-6 space-x-4">
               <Link<Route> classes={ get_classes(*route, Route::Dashboard) } to={Route::Dashboard}>{ "Dashboard" }</Link<Route>>
               <Link<Route> classes={ get_classes(*route, Route::Requests) } to={Route::Requests}>{ "Requests" }</Link<Route>>
               <Link<settings::SettingsRoute> classes={ get_classes(*route, Route::Settings) } to={settings::SettingsRoute::Filters}>{ "Settings" }</Link<settings::SettingsRoute>>
            </div>
          </div>
        </div>
      </div>
    </nav> };

    match route {
        Route::Dashboard => {
            set_title("Dashboard");

            html! { <>{navigation}<div class={"container mt-4 mb-10 mx-auto px-4 sm:px-6 lg:px-8"}> <dashboard::Dashboard /> </div></> }
        }
        Route::Requests => {
            set_title("Requests");
            html! { <>{navigation} <div class={"container mt-4 mb-10 mx-auto px-4 sm:px-6 lg:px-8"}> <requests::Requests /> </div></> }
        }
        Route::Settings => {
            html! {<>{navigation} <div class={"container mt-4 mb-10 mx-auto px-4 sm:px-6 lg:px-8"}> <Switch<settings::SettingsRoute> render={Switch::render(settings::switch_settings)} /> </div> </>}
        }
        Route::NotFound => {
            set_title("Not Found");
            html! { <>{navigation} <NotFound /></> }
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn set_title(title: &str) {
    gloo_utils::document().set_title(&format!("{} | Privaxy", title));
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<App>();
}

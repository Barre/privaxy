use crate::filters::Filters;
use crate::set_title;
use crate::settings_textarea::SettingsTextarea;
use yew::prelude::*;
use yew::{html, Html};
use yew_router::prelude::*;

#[derive(Clone, Copy, Routable, PartialEq)]
pub enum SettingsRoute {
    #[at("/settings/filters")]
    Filters,
    #[at("/settings/exclusions")]
    Exclusions,
    #[at("/settings/custom-filters")]
    CustomFilters,
}

pub fn switch_settings(route: &SettingsRoute) -> Html {
    fn get_classes(current_route: SettingsRoute, for_route_link: SettingsRoute) -> Classes {
        if current_route == for_route_link {
            classes!(
                "bg-gray-100",
                "text-gray-900",
                "flex",
                "items-center",
                "px-3",
                "py-2",
                "text-sm",
                "font-medium",
                "rounded-md"
            )
        } else {
            classes!(
                "text-gray-600",
                "hover:bg-gray-50",
                "hover:text-gray-900",
                "flex",
                "items-center",
                "px-3",
                "py-2",
                "text-sm",
                "font-medium",
                "rounded-md"
            )
        }
    }

    let content = match route {
        SettingsRoute::Filters => {
            set_title("Settings - Filters");

            html! { <Filters />}
        }
        SettingsRoute::Exclusions => {
            set_title("Settings - Exclusions");

            let description = html! {<div class="text-gray-600">
                    <p>
                        {"Exclusions are hosts or domains that are not passed through the MITM pipeline. "}
                        {"Excluded entries will be transparently tunneled."}
                    </p>
                </div>
            };
            let textarea_description = "Insert one entry per line";
            let set_resource_name = "set_exclusions";
            let get_resource_name = "get_exclusions";

            html! {<SettingsTextarea h1="Exclusions" {description} input_name="exclusions" {textarea_description} {set_resource_name} {get_resource_name} />}
        }
        SettingsRoute::CustomFilters => {
            set_title("Settings - Custom Filters");

            let description = html! {
                <p class="text-gray-600">
                    {"Insert EasyList compatible filters. Comment filters by prefixing lines with "} <span class="font-mono bg-gray-100">{"!"}</span>{"."}
                </p>
            };

            let textarea_description = "Insert one filter per line";
            let set_resource_name = "set_custom_filters".to_string();
            let get_resource_name = "get_custom_filters";

            html! {<SettingsTextarea h1="Custom Filters" {description} input_name="custom_filters" {textarea_description} {set_resource_name} {get_resource_name} />}
        }
    };

    html! {<div class="md:grid md:grid-cols-8">
    <nav class="space-y-1 mt-4 lg:col-span-1 sm:col-span-2" aria-label="Sidebar">
        <Link<SettingsRoute> classes={get_classes(*route, SettingsRoute::Filters)} to={SettingsRoute::Filters}> <span class="truncate">{ "Filters" }</span></Link<SettingsRoute>>
        <Link<SettingsRoute> classes={get_classes(*route, SettingsRoute::Exclusions)} to={SettingsRoute::Exclusions}> <span class="truncate">{ "Exclusions" }</span></Link<SettingsRoute>>
        <Link<SettingsRoute> classes={get_classes(*route, SettingsRoute::CustomFilters)} to={SettingsRoute::CustomFilters}> <span class="truncate">{ "Custom filters" }</span></Link<SettingsRoute>>
    </nav>
        <div class="container mx-auto px-4 sm:px-6 lg:px-8 mt-4 sm:col-span-6">{ content }</div>
    </div>
    }
}

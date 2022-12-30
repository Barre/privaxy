use crate::{save_button, submit_banner};
use serde::{Deserialize, Serialize};
use tauri_sys::tauri;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Callback, Component, Context, Html};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
enum FilterGroup {
    Default,
    Regional,
    Ads,
    Privacy,
    Malware,
    Social,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Filter {
    enabled: bool,
    title: String,
    group: FilterGroup,
    file_name: String,
}

#[derive(Serialize)]
pub struct FilterStatusChangeRequest {
    enabled: bool,
    file_name: String,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
pub struct FilterStatusChangeRequestPayload {
    filterStatusChangeRequest: Vec<FilterStatusChangeRequest>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FilterConfiguration(Vec<Filter>);

pub enum Message {
    Load,
    Display(FilterConfiguration),
    UpdateFilterSelection((String, bool)),
    Save,
    ChangesSaved,
}

pub struct Filters {
    filter_configuration: Option<FilterConfiguration>,
    filter_configuration_before_changes: Option<FilterConfiguration>,
    changes_saved: bool,
}

impl Filters {
    fn configuration_has_changed(&self) -> bool {
        self.filter_configuration != self.filter_configuration_before_changes
    }
}

impl Component for Filters {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Message::Load);

        Self {
            filter_configuration: None,
            filter_configuration_before_changes: None,
            changes_saved: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Display(filter_configuration) => {
                self.filter_configuration = Some(filter_configuration.clone());
                self.filter_configuration_before_changes = Some(filter_configuration);
            }
            Message::Load => {
                let message_callback = ctx.link().callback(|message: Message| message);

                spawn_local(async move {
                    let filter_configuration: FilterConfiguration =
                        tauri::invoke("get_filters_configuration", &())
                            .await
                            .unwrap();

                    message_callback.emit(Message::Display(filter_configuration))
                });
            }
            Message::Save => {
                if !self.configuration_has_changed() {
                    return false;
                }

                let request_body = self
                    .filter_configuration
                    .as_ref()
                    .unwrap()
                    .0
                    .iter()
                    .map(|filter| FilterStatusChangeRequest {
                        enabled: filter.enabled,
                        file_name: filter.file_name.clone(),
                    })
                    .collect::<Vec<_>>();

                let callback = ctx.link().callback(|message: Message| message);

                spawn_local(async move {
                    let _res = tauri::invoke::<_, FilterConfiguration>(
                        "change_filter_status",
                        &FilterStatusChangeRequestPayload {
                            filterStatusChangeRequest: request_body,
                        },
                    )
                    .await;
                    callback.emit(Message::ChangesSaved)
                });

                log::info!("Save")
            }
            Message::UpdateFilterSelection((filter_name, enabled)) => {
                self.changes_saved = false;

                self.filter_configuration
                    .as_mut()
                    .unwrap()
                    .0
                    .iter_mut()
                    .find(|filter| filter.file_name == filter_name)
                    .and_then(|filter| {
                        filter.enabled = enabled;

                        Some(filter)
                    });
            }
            Message::ChangesSaved => {
                self.changes_saved = true;
                self.filter_configuration_before_changes = self.filter_configuration.clone();
            }
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let save_button_state = if !self.configuration_has_changed() {
            save_button::SaveButtonState::Disabled
        } else {
            save_button::SaveButtonState::Enabled
        };

        let callback = ctx
            .link()
            .callback(|(filter_file_name, enabled): (String, bool)| {
                Message::UpdateFilterSelection((filter_file_name, enabled))
            });

        let save_callback = ctx.link().callback(|_| Message::Save);

        let render_category_filter = |filter: &Filter| {
            let filter_file_name = filter.file_name.clone();
            let filter_enabled = filter.enabled;
            let callback_clone = callback.clone();

            let checkbox_callback = Callback::from(move |_| {
                callback_clone.emit((filter_file_name.to_string(), !filter_enabled))
            });

            html! {
            <div class="relative flex items-start py-4">
                <div class="min-w-0 flex-1 text-sm">
                    <label for={filter.file_name.clone()} class="select-none">{&filter.title}</label>
                </div>
                <div class="ml-3 flex items-center h-5">
                    <input checked={filter.enabled} onchange={checkbox_callback} name={filter.file_name.clone()} type="checkbox"
                        class="focus:ring-blue-500 h-4 w-4 text-blue-600 border-gray-300 rounded" />
                </div>
            </div>
            }
        };

        let render_category = |category: FilterGroup, filters: &FilterConfiguration| {
            let category_name = format!("{:?}", category);
            let filters = filters.0.iter().filter(|filter| filter.group == category);

            html! {
            <fieldset class="mb-8">
                <legend class="text-lg font-medium text-gray-900">{category_name}</legend>
                <div class="mt-4 border-t border-b border-gray-200 divide-y divide-gray-200">
                    { for filters.into_iter().map(render_category_filter) }
                </div>
            </fieldset>
            }
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

        let title = html! {
            <div class="pt-1.5 mb-4">
                <h1 class="text-2xl font-bold text-gray-900">{ "Filters" }</h1>
            </div>
        };

        match &self.filter_configuration {
            Some(filter_configuration) => html! {
                <>
                    { title }
                    {success_banner}
                    <div class="mb-5">
                        <save_button::SaveButton state={save_button_state} onclick={save_callback} />
                    </div>
                    { render_category(FilterGroup::Default, filter_configuration) }
                    { render_category(FilterGroup::Ads, filter_configuration) }
                    { render_category(FilterGroup::Privacy, filter_configuration) }
                    { render_category(FilterGroup::Malware, filter_configuration) }
                    { render_category(FilterGroup::Social, filter_configuration) }
                    { render_category(FilterGroup::Regional, filter_configuration) }
                </>
            },
            // This realistically loads way too fast for a loader to be useful. Adding one would just add
            // unwanted flickering.
            None => html! {{ title }},
        }
    }
}

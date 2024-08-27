use crate::wrappers::{Error, Info, StyledView, TitleBar};
use experiences_navigator_lib::{
    api::api_request,
    navigator::{Navigator, NavigatorOutput},
};
use leptos::*;
use leptos_router::*;
use shared::{
    standalone_experience_types::types::ExperiencesHostname,
    timeline::{frontend::plugin_manager::PluginManager, types::api::TimelineHostname},
};

mod experience;
mod wrappers;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <MainView/> }
    })
}

#[component]
fn MainView() -> impl IntoView {
    provide_context(ExperiencesHostname(leptos::window().origin()));
    view! {
        <Router>
            <Routes>
                <Route path="/experience/:id" view=ExperienceView/>
                <Route path="/" view=Redirect/>
                <Route path="*not_found" view=NotFound/>
            </Routes>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <StyledView>
            <TitleBar subtitle=Some("404 - Not Found".to_string())/>
            <div class="errorWrapper">Was unable to find the page you are looking for.</div>
        </StyledView>
    }
}

#[component]
fn Redirect() -> impl IntoView {
    todo!("Automatically redirect to the last page the admin navigated to");
    use_navigate()("/experience/", NavigateOptions::default());
    view! { <div class="intoWrapper">"Redirecting"</div> }
}

#[component]
fn ExperienceView() -> impl IntoView {
    let params = use_params_map();
    let experience_id = Signal::derive(move || params().get("id").cloned().unwrap()); //this experience component is attached to a route where this is defined

    let timeline_url_error = create_resource(
        || {},
        |_| async {
            match api_request::<String, _>("/timeline_url", &()).await {
                Ok(v) => {
                    provide_context(TimelineHostname(v));
                    None
                }
                Err(e) => Some(e),
            }
        },
    );

    let plugin_manager = create_action(|_: &()| async { PluginManager::new().await });
    plugin_manager.dispatch(());

    view! {
        <StyledView>
            <Suspense fallback=move || {view! {
                <Info>Loading</Info>
            }}>
                {move || {
                    timeline_url_error()
                        .map(|timeline_url_error| {
                            match timeline_url_error {
                                None => {
                                    view! {
                                        <TitleBar/>
                                        <Navigator
                                            experience=experience_id
                                            navigate=create_signal(
                                                    NavigatorOutput::Callback(
                                                        Callback::new(|id| {
                                                            {
                                                                use_navigate()(
                                                                    &format!("/experience/{}", id),
                                                                    NavigateOptions::default(),
                                                                )
                                                            }
                                                        }),
                                                    ),
                                                )
                                                .0
                                        />
                                        {move || match plugin_manager.value()() {
                                            Some(v) => {
                                                provide_context(v);
                                                view! {
                                                    <experience::Experience id=experience_id></experience::Experience>
                                                }
                                            }
                                            None => {
                                                view! { <Info>Loading</Info> }
                                            }
                                        }}
                                    }
                                        .into_view()
                                }
                                Some(e) => view! { <Error>Error loading Timeline Url: {e.to_string()}</Error> },
                            }
                        })
                }}
            </Suspense>
        </StyledView>
    }
}

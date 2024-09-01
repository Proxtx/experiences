#![feature(type_alias_impl_trait)]
use {
    experiences_navigator_lib::{
        api::api_request,
        navigator::{Navigator, NavigatorOutput},
        wrappers::{Error, Info, StyledView, TitleBar},
    },
    leptos::*,
    leptos_router::*,
    shared::{
        standalone_experience_types::types::ExperiencesHostname,
        timeline::{
            frontend::{
                events_display::DisplayWithDay, plugin_manager::PluginManager, wrappers::Login,
            },
            types::api::TimelineHostname,
        },
    },
};

mod experience;

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
    let (update_authentication, write_update_authentication) = create_signal(0);

    let error = create_resource(update_authentication, |_| async move {
        match api_request::<String, _>("/navigator/position", &()).await {
            Err(e) => Some(e),
            Ok(v) => {
                use_navigate()(&format!("/experience/{}", v), NavigateOptions::default());
                None
            }
        }
    });
    view! {
        <StyledView>
            <TitleBar/>
            <Suspense fallback=move || {
                view! { <Info>Navigating</Info> }
            }>

                {move || {
                    error
                        .map(|v| {
                            match v {
                                Some(e) => {
                                    let e = e.clone();
                                    view! {
                                        <Error>
                                            Error loading Navigator Position: {e.to_string()}
                                        </Error>
                                        <Login update_authentication=write_update_authentication/>
                                    }
                                        .into_view()
                                }
                                None => {
                                    view! { <Info>Redirecting</Info> }
                                }
                            }
                        })
                }}

            </Suspense>
        </StyledView>
    }
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

    let (navigator_expanded, write_navigator_expanded) = create_signal(true);

    let display_width_day = DisplayWithDay(true);
    provide_context(display_width_day);

    view! {
        <StyledView>
            <Suspense fallback=move || {
                view! { <Info>Loading</Info> }
            }>
                {move || {
                    timeline_url_error()
                        .map(|timeline_url_error| {
                            match timeline_url_error {
                                None => {
                                    view! {
                                        <TitleBar/>
                                        <div on:click=move |_| write_navigator_expanded(true)>
                                            <Navigator
                                                experience=experience_id
                                                options=true
                                                expanded=navigator_expanded
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
                                        </div>
                                        {move || match plugin_manager.value()() {
                                            Some(v) => {
                                                provide_context(v);
                                                view! {
                                                    <div
                                                        style="height: 100%; overflow: auto;display: flex;flex-direction: column;"
                                                        on:click=move |_| { write_navigator_expanded(false) }
                                                    >
                                                        <experience::Experience id=experience_id></experience::Experience>
                                                    </div>
                                                }
                                                    .into_view()
                                            }
                                            None => view! { <Info>Loading</Info> }.into_view(),
                                        }}
                                    }
                                        .into_view()
                                }
                                Some(e) => {
                                    view! {
                                        <Error>Error loading Timeline Url: {e.to_string()}</Error>
                                    }
                                }
                            }
                        })
                }}

            </Suspense>
        </StyledView>
    }
}

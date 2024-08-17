use crate::wrappers::{Info, StyledView, TitleBar};
use leptos::*;
use leptos_router::*;
use navigator::StandaloneNavigator;
use shared::timeline::frontend::plugin_manager::PluginManager;

mod experience;
mod navigator;
mod wrappers;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <MainView/> }
    })
}

#[component]
fn MainView() -> impl IntoView {
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

    let plugin_manager = create_action(|_: &()| async { PluginManager::new().await });
    plugin_manager.dispatch(());

    view! {
        <StyledView>
            <TitleBar/>
            <StandaloneNavigator/>
            {move || match plugin_manager.value()() {
                Some(v) => {
                    provide_context(v);
                    view! { <experience::Experience id=experience_id></experience::Experience> }
                }
                None => {
                    view! { <Info>Loading</Info> }
                }
            }}

        </StyledView>
    }
}

use crate::wrappers::{StyledView, TitleBar};
use leptos::*;
use leptos_router::*;

mod experience;
mod wrappers;

fn main() {
    mount_to_body(|| {
        view! {
            <MainView />
        }
    })
}

#[component]
fn MainView() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/experience/:id" view=ExperienceView />
                <Route path="/" view=Redirect />
                <Route path="*not_found" view=NotFound />
            </Routes>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <StyledView>
            <TitleBar subtitle=Some("404 - Not Found".to_string()) />
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

    view! {
        <TitleBar />
        <experience::Experience id=experience_id/>
    }
}

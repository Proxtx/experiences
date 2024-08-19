use std::collections::HashMap;

use crate::wrappers::Error;
use crate::wrappers::Info;
use leptos::*;
use shared::timeline::frontend::api::api_request;
use shared::timeline::frontend::events_display::EventsViewer;
use shared::timeline::frontend::plugin_manager::PluginManager;
use shared::types::Experience;
use shared::types::ExperienceEvent;

#[component]
pub fn Experience(#[prop(into)] id: MaybeSignal<String>) -> impl IntoView {
    let experience = create_resource(id, |id: String| async move {
        api_request::<Experience, _>(&format!("/experience/{}", id), &()).await
    });

    view! {
        <Suspense fallback=move || {
            view! { <Info>Loading</Info> }
        }>

            {move || {
                experience()
                    .map(|experience| match experience {
                        Ok(v) => view! { <ExperienceView experience=v/> }.into_view(),
                        Err(e) => {
                            view! {
                                <Error>{move || format!("Error loading Experience: {}", e)}</Error>
                            }
                                .into_view()
                        }
                    })
            }}

        </Suspense>
    }
}

#[component]
pub fn ExperienceView(#[prop(into)] experience: MaybeSignal<Experience>) -> impl IntoView {
    let plugin_manager = use_context::<PluginManager>()
        .expect("Plugin manager was not provided as context! Not good, not recoverable.");

    type GenTypeParam2 = fn(ExperienceEvent) -> View;

    let t: GenTypeParam2 = |e: ExperienceEvent| view! { <h1>hi</h1> }.into_view();

    view! {
        <EventsViewer<ExperienceEvent, GenTypeParam2>
            events=Signal::derive(move || experience().events)
            plugin_manager
            slide_over=Some(t)
        />
    }
}

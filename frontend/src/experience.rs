use std::collections::HashMap;

use experiences_navigator_lib::{
    api::api_request,
    navigator::StandaloneNavigator,
    wrappers::{Band, Error, Info, StyledView},
};
use leptos::*;
use shared::timeline::frontend::events_display::EventsViewer;
use shared::timeline::frontend::plugin_manager::PluginManager;
use shared::types::Experience;
use shared::types::ExperienceEvent;
use std::sync::Arc;

#[component]
pub fn Experience(#[prop(into)] id: MaybeSignal<String>) -> impl IntoView {
    let experience = create_resource(id.clone(), |id: String| async move {
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

    type GenTypeParam2 = fn(ExperienceEvent, Box<dyn Fn()>) -> View;

    let t: GenTypeParam2 = |event: ExperienceEvent, close_callback| {
        let selected_experience = create_rw_signal(None);
        let close_callback = Arc::new(close_callback);
        let close_callback_2 = close_callback.clone();

        view! {
            <StyledView>
                <Band click=Callback::new(move |_| {
                    close_callback_2();
                })>
                    <b>Close</b>
                </Band>
                <Band color="var(--accentColor2)".to_string()>
                    <b>Delete</b>
                </Band>
                <Band color="var(--accentColor1)".to_string()>
                    <b>Favorite</b>
                </Band>
                <StandaloneNavigator selected_experience=selected_experience/>
                <Band click=Callback::new(move |_| {
                    spawn_local({
                        let close_callback = close_callback.clone();
                        let selected_experience = selected_experience();
                        let event = event.clone();
                        async move {
                            close_callback();
                            if let Err(e) = experiences_navigator_lib::api::api_request::<
                                String,
                                _,
                            >(
                                    &format!(
                                        "/experience/{}/append_event",
                                        selected_experience.unwrap(),
                                    ),
                                    &(event.plugin, event.event),
                                )
                                .await
                            {
                                window()
                                    .alert_with_message(
                                        &format!("Unable to append event to experience: {}", e),
                                    )
                                    .unwrap();
                            }
                        }
                    })
                })>
                    <b>Insert</b>
                </Band>
            </StyledView>
        }
        .into_view()
    };

    view! {
        <EventsViewer<ExperienceEvent, GenTypeParam2>
            events=Signal::derive(move || experience().events)
            plugin_manager
            slide_over=Some(t)
        />
    }
}

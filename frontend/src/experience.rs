use std::collections::HashMap;

use {
    experiences_navigator_lib::{
        api::api_request,
        navigator::StandaloneNavigator,
        wrappers::{Band, Error, Info, StyledView},
    },
    leptos::*,
    shared::{
        timeline::{
            frontend::{events_display::EventsViewer, plugin_manager::PluginManager},
            types::api::AvailablePlugins,
        },
        types::{Experience, ExperienceEvent, PluginExperienceEvent},
    },
    std::sync::Arc,
};

#[component]
pub fn Experience(#[prop(into)] id: MaybeSignal<String>) -> impl IntoView {
    let experience = create_resource(id.clone(), |id: String| async move {
        api_request::<Experience, _>(&format!("/experience/{}", id), &())
            .await
            .map(|v| (id, v))
    });

    view! {
        <Suspense fallback=move || {
            view! { <Info>Loading</Info> }
        }>
            {move || {
                experience()
                    .map(|experience_loaded| match experience_loaded {
                        Ok((id, v)) => {
                            view! {
                                <ExperienceView
                                    id
                                    experience=v
                                    reload=Callback::new(move |_| {
                                        experience.refetch();
                                    })
                                />
                            }
                                .into_view()
                        }
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
pub fn ExperienceView(
    #[prop(into)] id: MaybeSignal<String>,
    #[prop(into)] experience: MaybeSignal<Experience>,
    reload: Callback<(), ()>,
) -> impl IntoView {
    let plugin_manager = use_context::<PluginManager>()
        .expect("Plugin manager was not provided as context! Not good, not recoverable.");

    type GenTypeParam2 = impl Fn(PluginExperienceEvent, Box<dyn Fn()>) -> View + Clone;

    let t: GenTypeParam2 = move |event: PluginExperienceEvent, close_callback| {
        let selected_experience = create_rw_signal(None);
        let close_callback = Arc::new(close_callback);
        let close_callback_2 = close_callback.clone();
        let close_callback_3 = close_callback.clone();
        let close_callback_4 = close_callback.clone();
        let event_2 = event.clone();
        let event_3 = event.clone();
        let id = id.clone();
        let id_2 = id.clone();

        let (expanded, write_expanded) = create_signal(false);

        view! {
            <StyledView>
                <Band click=Callback::new(move |_| {
                    close_callback_2();
                })>
                    <b>Close</b>
                </Band>
                <div class="optionsBand" style="display: flex;flex-direction: row">
                    <Band
                        color="var(--accentColor2)".to_string()
                        click=Callback::new(move |_| {
                            spawn_local({
                                let id = id();
                                let close_callback = close_callback_4.clone();
                                let event = event_3.clone();
                                async move {
                                    close_callback();
                                    if let Err(e) = experiences_navigator_lib::api::api_request::<
                                        Option<(AvailablePlugins, ExperienceEvent)>,
                                        _,
                                    >(&format!("/experience/{}/delete", id), &event.1.id)
                                        .await
                                    {
                                        window()
                                            .alert_with_message(
                                                &format!("Unable to delete event: {}", e),
                                            )
                                            .unwrap();
                                    }
                                    reload(());
                                }
                            });
                        })
                    >

                        <img src="/icons/delete.svg"/>
                    </Band>
                    <Band
                        color="var(--accentColor1)".to_string()
                        click=Callback::new(move |_| {
                            spawn_local({
                                let id = id_2();
                                let close_callback = close_callback_3.clone();
                                let event = event_2.clone();
                                async move {
                                    close_callback();
                                    if let Err(e) = experiences_navigator_lib::api::api_request::<
                                        (),
                                        _,
                                    >(
                                            &format!("/experience/{}/favorite", id),
                                            &shared::types::FavoriteRequest {
                                                event_id: event.1.id,
                                                favorite: !event.1.favorite,
                                            },
                                        )
                                        .await
                                    {
                                        window()
                                            .alert_with_message(
                                                &format!("Unable to (un-)favorite event: {}", e),
                                            )
                                            .unwrap();
                                    }
                                    reload(());
                                }
                            });
                        })
                    >

                        <img src=move || {
                            if event.1.favorite {
                                "/icons/starFilled.svg"
                            } else {
                                "/icons/starOutline.svg"
                            }
                        }/>
                    </Band>
                </div>
                <div on:click=move |_| write_expanded(true)>
                    <StandaloneNavigator expanded selected_experience=selected_experience/>
                </div>
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
                                    &(event.0, event.1.event),
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
        <EventsViewer<PluginExperienceEvent, GenTypeParam2>
            events=Signal::derive(move || experience().events.into_iter().map(|(plg, event)| {
                let mut event = event.into_iter().map(|event| {PluginExperienceEvent(plg.clone(), event)}).collect::<Vec<_>>();
                event.sort_by(|e1, e2| {e1.1.event.time.cmp(&e2.1.event.time)});
                (plg.clone(), event)}).collect::<HashMap<AvailablePlugins, Vec<PluginExperienceEvent>>>())
            plugin_manager
            slide_over=Some(t)
        />
    }
}

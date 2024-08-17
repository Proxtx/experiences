use crate::wrappers::{Error, Info};
use leptos::*;
use leptos_use::*;
use shared::{timeline::frontend::api::api_request, types::ExperienceConnectionResponse};
use stylers::style;

#[component]
pub fn StandaloneNavigator(
    #[prop(into, default=create_rw_signal(None))] selected_experience: RwSignal<Option<String>>,
) -> impl IntoView {
    let selected_experience_internal = create_memo(move |_| selected_experience());
    create_effect(move |_| selected_experience.set(selected_experience_internal()));

    let (global_position_error, write_global_position_error) = create_signal(None);

    let global_position = create_action(move |_| async move {
        if selected_experience_internal.get_untracked().is_none() {
            match api_request("/navigator/position", &()).await {
                Err(e) => write_global_position_error(Some(e)),
                Ok(v) => selected_experience.set(v),
            }
        }
    });

    create_effect(move |_| {
        if selected_experience_internal().is_none() {
            global_position.dispatch(());
        }
    });

    view! {
        {move || match global_position_error() {
            Some(e) => {
                view! { <Error>Error loading navigator position: {move || e.to_string()}</Error> }
                    .into_view()
            }
            None => {
                view! {
                    {move || match selected_experience_internal() {
                        Some(v) => {
                            let experience = create_rw_signal(v.clone());
                            create_effect(move |_| {
                                selected_experience.set(Some(experience().clone()))
                            });
                            view! { <Navigator experience/> }.into_view()
                        }
                        None => view! { <Info>Loading</Info> }.into_view(),
                    }}
                }
                    .into_view()
            }
        }}
    }
}

#[component]
pub fn Navigator(
    #[prop(into)] experience: RwSignal<String>,
    #[prop(into, default=true.into())] expanded: MaybeSignal<bool>,
) -> impl IntoView {
    let style = style! {
        .navigator_wrapper {
            width: 100%;
            height: 300px;
            background-color: var(--accentColor3Light);
            transition: 0.5s;
        }

        .collapsed {
            height: 200px;
        }

        .loading {
            height: 0px;
        }
    };

    let (read_width, write_width) = create_signal(0.0);
    let (read_height, write_height) = create_signal(0.0);

    let wrapper = create_node_ref();
    use_resize_observer(wrapper, move |entries, _observer| {
        let rect = entries[0].content_rect();
        write_width.set(rect.width());
        write_height.set(rect.height());
    });

    let connections = create_resource(experience, |experience| async move {
        api_request::<ExperienceConnectionResponse, _>(&format!("/navigator/{}", experience), &())
            .await
    });

    view! { class=style,
        <div
            ref=wrapper
            class="navigator_wrapper"
            class:collapsed=move || !expanded()
            class:loading=move || connections().is_none()
        ></div>
        <div style:display=move || connections().map(|_| "none").unwrap_or("block")>
            <Info>{move || serde_json::to_string(&connections())}</Info>
        </div>
    }
}

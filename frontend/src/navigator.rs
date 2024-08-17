use ::core::f64;

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
            position: relative;
        }

        .collapsed {
            height: 200px;
        }

        .loading {
            height: 0px;
        }

        .experience_wrap {
            transform: translate(-50%, -50%);
            position: absolute;
            z-index: 2;
        }

        .centerExperienceCard {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            z-index: 2;
        }

        .connection {
            border: 2px solid var(--accentColor3);
            position: absolute;
            top: 50%;
            left: 50%;
            transform-origin: center left;
            z-index: 1;
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
        >
            <Suspense fallback=move || {
                view! { <Info>Loading</Info> }
            }>
                {move || {
                    connections()
                        .map(|connections| {
                            view! {
                                {move || match connections.clone() {
                                    Ok(connections) => {
                                        let view = connections
                                            .connections
                                            .iter()
                                            .enumerate()
                                            .map(|(pos, connection)| {
                                                let deg = pos as f64 / connections.connections.len() as f64
                                                    * 360.0 + 270.0 + 0.0;
                                                let rad = deg * f64::consts::PI / 180.0;
                                                let x = move || {
                                                    let width = read_width();
                                                    width / 2.0 + (width / 8.0 * 3.0) * rad.cos()
                                                };
                                                let y = move || {
                                                    let height = read_height();
                                                    height / 2.0 + (height / 8.0 * 2.5) * rad.sin()
                                                };
                                                let connection_len = move || {
                                                    let height = read_height();
                                                    let width = read_width();
                                                    ((height / 2.0 - y()).powf(2.0)
                                                        + (width / 2.0 - x()).powf(2.0))
                                                        .sqrt()
                                                };
                                                view! { class=style,
                                                    <div
                                                        class="experience_wrap"
                                                        style:left=move || format!("{}px", x())
                                                        style:top=move || format!("{}px", y())
                                                    >
                                                        <ExperienceCard
                                                            name=connection.name.clone()
                                                            id=connection.id.clone()
                                                        />
                                                    </div>
                                                    <div
                                                        class="connection"
                                                        style:width=move || format!("{}px", connection_len())
                                                        style:transform=move || { format!("rotate({}deg)", deg) }
                                                    ></div>
                                                }
                                            })
                                            .collect_view();
                                        view! { class=style,
                                            {view}
                                            <div class="centerExperienceCard">
                                                <ExperienceCard
                                                    name=connections.experience_name.clone()
                                                    id=experience()
                                                    enlarge=true
                                                />
                                            </div>
                                        }
                                            .into_view()
                                    }
                                    Err(e) => {
                                        let e = e.clone();
                                        view! {
                                            <Error>
                                                Error loading connected experiences :
                                                {move || e.to_string()}
                                            </Error>
                                        }
                                            .into_view()
                                    }
                                }}
                            }
                                .into_view()
                        })
                }}

            </Suspense>

        </div>
        <div style:display=move || connections().map(|_| "none").unwrap_or("block")>
            <Info>{move || serde_json::to_string(&connections())}</Info>
        </div>
    }
}

#[component]
pub fn ExperienceCard(
    #[prop(into)] name: MaybeSignal<String>,
    #[prop(into)] id: MaybeSignal<String>,
    #[prop(into, default=false.into())] enlarge: MaybeSignal<bool>,
    #[prop(into, default=false.into())] focus: MaybeSignal<bool>,
    #[prop(into, default=Callback::new(|_| {}))] click: Callback<web_sys::MouseEvent, ()>,
) -> impl IntoView {
    let style = style! {
        img {
            width: 100%;
            border-radius: 5px;
        }

        .innerWrap {
            width: 50px;
            display: flex;
            align-items: center;
            flex-direction: column;
            border: 3px solid var(--accentColor3);
            border-radius: 5px;
            background-color: var(--accentColor3);
        }

        .textWrap {
            width: 100%;
            color: var(--lightColor);
            word-wrap: break-word;
            text-align: center;
            font-size: 80%;
        }

        .enlarge {
            width: 70px;
        }
    };

    view! { class=style,
        <div class="innerWrap" class:enlarge=enlarge on:click=click>
            <img src="/icons/logo.png"/>
            <a class="textWrap">{name}</a>
        </div>
    }
}

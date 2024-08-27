use ::core::f64;

use crate::api::{api_request, relative_url};
use experiences_types_lib::types::ExperienceConnectionResponse;
use leptos::*;
use leptos_use::*;
use serde::Deserialize;
use stylers::style;

#[component]
pub fn StandaloneNavigator(
    #[prop(into, default=create_rw_signal(None))] selected_experience: RwSignal<Option<String>>,
) -> impl IntoView {
    let selected_experience_internal = create_memo(move |_| selected_experience());
    create_effect(move |_| selected_experience.set(selected_experience_internal()));

    let global_position_error = create_resource(
        selected_experience_internal,
        move |selected_experience_internal| async move {
            if selected_experience_internal.is_none() {
                match api_request("/navigator/position", &()).await {
                    Err(e) => Some(e),
                    Ok(v) => {
                        selected_experience.set(v);
                        None
                    }
                }
            } else {
                None
            }
        },
    );

    view! {
        <Suspense fallback=move || {
            view! { <Info>Loading</Info> }
        }>
            {move || {
                global_position_error()
                    .map(|positioning_error| {
                        match positioning_error {
                            None => {
                                view! {
                                    {move || match selected_experience_internal() {
                                        Some(v) => {
                                            let (experience, write_experience) = create_signal(
                                                v.clone(),
                                            );
                                            create_effect(move |_| {
                                                selected_experience.set(Some(experience().clone()))
                                            });
                                            view! {
                                                <Navigator
                                                    experience
                                                    navigate=create_signal(
                                                            NavigatorOutput::Signal(write_experience),
                                                        )
                                                        .0
                                                />
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
                                    <Error>
                                        Error loading navigator position: {move || e.to_string()}
                                    </Error>
                                }
                                    .into_view()
                            }
                        }
                    })
            }}

        </Suspense>
    }
}

#[derive(Clone, Debug)]
pub enum NavigatorOutput {
    Callback(Callback<String>),
    Signal(WriteSignal<String>),
    None,
}

impl NavigatorOutput {
    pub fn output(&self, output: String) {
        match self {
            NavigatorOutput::Callback(v) => v(output),
            NavigatorOutput::None => {}
            NavigatorOutput::Signal(v) => v.set(output),
        }
    }
}

#[component]
pub fn Navigator(
    #[prop(into)] experience: Signal<String>,
    #[prop(into, default=true.into())] expanded: MaybeSignal<bool>,
    #[prop(into, default=create_signal(NavigatorOutput::None).0.into())] navigate: Signal<
        NavigatorOutput,
    >,
) -> impl IntoView {
    let style = style! {
        .navigator_wrapper {
            width: 100%;
            height: 300px;
            background-color: var(--accentColor3Light);
            position: relative;
            overflow: hidden;
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
        <div ref=wrapper class="navigator_wrapper" class:collapsed=move || !expanded()>
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
                                                let connection_id = connection.id.clone();
                                                view! { class=style,
                                                    <div
                                                        class="experience_wrap"
                                                        style:left=move || format!("{}px", x())
                                                        style:top=move || format!("{}px", y())
                                                    >
                                                        <ExperienceCard
                                                            name=connection.name.clone()
                                                            id=connection.id.clone()
                                                            click=Callback::new(move |_e: web_sys::MouseEvent| {
                                                                navigate().output(connection_id.clone());
                                                            })
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
            <img src=move || {relative_url("/icons/logo.png").unwrap().to_string()}/>
            <a class="textWrap">{name}</a>
        </div>
    }
}

#[component]
pub fn Info(children: Children) -> impl IntoView {
    view! { <div class="infoWrapper">{children()}</div> }
}

#[component]
pub fn Error(children: Children) -> impl IntoView {
    view! { <div class="errorWrapper">{children()}</div> }
}

use {
    crate::{
        api::{api_request, relative_url},
        wrappers::Band,
    },
    ::core::f64,
    experiences_types_lib::types::ExperienceConnectionResponse,
    leptos::*,
    leptos_use::*,
    stylers::style,
};

#[component]
pub fn StandaloneNavigator(
    #[prop(into, default=create_rw_signal(None))] selected_experience: RwSignal<Option<String>>,
    #[prop(into, default=true.into())] expanded: MaybeSignal<bool>,
) -> impl IntoView {
    //WTF FIX THIS YOU MONKE

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
                                            let (experience, _write_experience) = create_signal(
                                                v.clone(),
                                            );
                                            view! {
                                                <Navigator
                                                    expanded
                                                    experience
                                                    navigate=create_signal(
                                                            NavigatorOutput::Callback(
                                                                Callback::new(move |v| { selected_experience.set(Some(v)) }),
                                                            ),
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
    #[prop(into, default=false.into())] options: MaybeSignal<bool>,
) -> impl IntoView {
    let style = style! {
        .navigator_wrapper {
            width: 100%;
            height: 300px;
            background-color: var(--accentColor3Light);
            position: relative;
            overflow: hidden;
            transition: 0.5s;
        }

        .collapsed {
            height: 150px;
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

        .connectedExperiencesWrapper {
            position: absolute;
            top: 50%;
            transform: translateY(-50%);
            left: 0;
            width: 100%;
            height: 100%;
            transition: 1s;
            opacity: 1;
        }

        .collapsedConnections {
            height: 0%;
            opacity: 0;
        }

        .collapsedConnections > * {
            pointer-events: none;
        }
    };

    let expanded = create_memo(move |_| expanded());

    let (read_width, write_width) = create_signal(0.0);
    let (read_height, write_height) = create_signal(0.0);

    let connections = create_resource(experience, |experience| async move {
        api_request::<ExperienceConnectionResponse, _>(&format!("/navigator/{}", experience), &())
            .await
    });

    view! { class=style,
        <div class="navigator_wrapper" class:collapsed=move || !expanded()>
            <Suspense fallback=move || {
                view! { <Info>Loading</Info> }
            }>
                {move || {
                    connections()
                        .map(|connections| {
                            view! {
                                {move || match connections.clone() {
                                    Ok(connections) => {
                                        let wrapper = create_node_ref();
                                        use_resize_observer(wrapper, move |entries, _observer| {
                                            let rect = entries[0].content_rect();
                                            write_width.set(rect.width());
                                            write_height.set(rect.height());
                                        });

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
                                                let connection_rotation = move || {
                                                    let height = read_height();
                                                    let width = read_width();
                                                    let relative_y = y() - height / 2.0;
                                                    let relative_x = x() - width / 2.0;

                                                    let mut res = (-1.0 * (relative_y) / ((relative_x).powi(2) + (relative_y).powi(2)).sqrt()).acos() * (180. as f64 / f64::consts::PI);
                                                    if relative_x < 0. {res += (180. - res) * 2.}
                                                    res - 90.
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
                                                        style:transform=move || { format!("rotate({}deg)", connection_rotation()) }
                                                    ></div>
                                                }
                                            })
                                            .collect_view();
                                        view! { class=style,
                                            <div class="connectedExperiencesWrapper" ref=wrapper class:collapsedConnections = move || {!expanded()}>
                                                {view}
                                            </div>
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
        <Suspense fallback=move || view! {<Info>Loading</Info>}>
        {
            move || {
                if options() && expanded() {
                    match connections() {
                        Some(loading_result) => match loading_result {
                            Ok(connections) => {
                                let (connections, write_connections) = create_signal(connections);
                                let (expanded, write_expanded) = create_signal(false);
                                view! {
                                    <Band click=Callback::new(move |_| {write_expanded.update(|v| *v = !*v)})>
                                        <img src=relative_url("/icons/arrow.svg").unwrap().to_string() style="position:absolute; left: var(--contentSpacing); top: 50%;transition: 100ms" style:transform=move || {
                                            if expanded() {
                                                "translateY(-50%) rotate(90deg)"
                                            }
                                            else {
                                                "translateY(-50%) rotate(0deg)"
                                            }
                                        }/>
                                        {move || {connections().experience_name}}
                                    </Band>
                                    <div style:display=move || if expanded() {"block"} else {"none"}>
                                        <Band color="var(--accentColor1)"
                                        click=Callback::new(move |_| {
                                            let experience_id = experience();
                                            let new_connection_status = !connections().public;
                                            write_connections.update(|v| v.public = new_connection_status);
                                            spawn_local(async move {
                                                if let Err(e) = api_request::<(), _>(&format!("/experience/{}/visibility", experience_id), &new_connection_status).await {
                                                    window().alert_with_message(&format!("Unable to change visibility: {}", e)).unwrap();
                                                }
                                            })
                                        })
                                        >
                                            <img src=move|| {
                                                if connections().public {
                                                    relative_url("/icons/public.svg").unwrap().to_string()
                                                }else {
                                                    relative_url("/icons/private.svg").unwrap().to_string()
                                                }}/>
                                        </Band>
                                    </div>
                            }.into_view()}
                            Err(e) => {
                                view! {
                                    <Error>
                                        Error loading Navigator: {e.to_string()}
                                    </Error>
                                }.into_view()
                            }
                        }
                        None => {
                            view! {
                                <Info>Loading</Info>
                            }.into_view()
                        }
                    }
                }
                else {
                    ().into_view()
                }
            }
        }
        </Suspense>
    }
}

#[component]
pub fn ExperienceCard(
    #[prop(into)] name: MaybeSignal<String>,
    #[prop(into)] id: MaybeSignal<String>,
    #[prop(into, default=false.into())] enlarge: MaybeSignal<bool>,
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
            <img src=move || { relative_url(&format!("/api/experience/{}/cover/small", id())).unwrap().to_string() }/>
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

use {leptos::*, stylers::style, web_sys::MouseEvent};

#[component]
pub fn TitleBar(
    #[prop(into, default=None.into())] subtitle: MaybeSignal<Option<String>>,
    #[prop(into, default=Callback::new(|_| {}))] subtitle_click_callback: Callback<MouseEvent, ()>,
) -> impl IntoView {
    let style = style! {
        .wrapper {
            width: 100%;
            display: flex;
            align-items: center;
            flex-direction: column;
            background-color: var(--darkColor);
            --padding: calc(var(--contentSpacing) * 3.5);
            padding-top: var(--padding);
            padding-bottom: var(--padding);
            gap: calc(var(--contentSpacing) * 1.5);
        }

        .titleWrapper {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: center;
            gap: var(--contentSpacing);
        }

        .logo {
            height: 40px;
            transition: 500ms;
            transform: rotate(0deg);
        }

        .subtitle {
            color: var(--accentColor1);
            text-decoration: none;
        }
    };

    view! { class=style,
        <div class="wrapper">
            <div class="titleWrapper">
                <img
                    src="/icons/logoTransparent.svg"
                    class="logo"
                    on:click=|v| {
                        event_target::<web_sys::HtmlElement>(&v)
                            .style()
                            .set_property("transform", "rotate(360deg)")
                            .unwrap();
                        let _ = leptos::window().location().reload();
                    }
                />

                <h1 class="title">Experiences</h1>
            </div>
            {move || match subtitle() {
                Some(v) => {
                    view! { class=style,
                        <a href="javascript:" class="subtitle" on:click=subtitle_click_callback>
                            {v}
                        </a>
                    }
                        .into_view()
                }
                None => view! {}.into_view(),
            }}

        </div>
    }
}

#[component]
pub fn StyledView(children: Children) -> impl IntoView {
    let stylers_class = style! {
        .view {
            display: flex;
            flex-direction: column;
            width: 100%;
            height: 100%;
        }
    };
    view! { class=stylers_class, <div class="view">{children()}</div> }
}

#[component]
pub fn Info(children: Children) -> impl IntoView {
    view! { <div class="infoWrapper">{children()}</div> }
}

#[component]
pub fn Error(children: Children) -> impl IntoView {
    view! { <div class="errorWrapper">{children()}</div> }
}

#[component]
pub fn Band(
    children: Children,
    #[prop(into, default=create_signal("var(--accentColor3)".to_string()).0.into())]
    color: MaybeSignal<String>,
    #[prop(into, default=Callback::new(|_|{}))] click: Callback<MouseEvent, ()>,
) -> impl IntoView {
    let style = style! {
        .band {
            padding: var(--contentSpacing);
            box-sizing: border-box;
            color: var(--lightColor);
            width: 100%;
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: center;
            position: relative;
        }
    };
    view! { class=style,
        <div class="band" style:background-color=color on:click=click role="button">
            {children()}
        </div>
    }
}

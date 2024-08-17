use crate::wrappers::{Error, Info};
use leptos::*;
use shared::timeline::frontend::api::api_request;

#[component]
pub fn StandaloneNavigator(
    #[prop(into, default=create_rw_signal(None))] selected_experience: RwSignal<Option<String>>,
) -> impl IntoView {
    let selected_experience_async =
        create_resource(selected_experience, |selected_experience| async move {
            logging::log!("Reloading selected experience");
            match selected_experience {
                Some(v) => Ok(v),
                None => api_request("/navigator/position", &()).await,
            }
        });

    view! {
        <Suspense fallback=move || {
            view! { <Info>Loading</Info> }
        }>

            {move || {
                selected_experience_async
                    .map(|v| {
                        match v {
                            Ok(v) => {
                                let experience = create_rw_signal(v.clone());
                                create_effect(move |_| {
                                    selected_experience.set(Some(experience().clone()))
                                });
                                view! { <Navigator experience/> }.into_view()
                            }
                            Err(e) => {
                                let e = e.clone();
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

#[component]
pub fn Navigator(#[prop(into)] experience: RwSignal<String>) -> impl IntoView {
    view! { {experience} }
}

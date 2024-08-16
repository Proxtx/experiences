use leptos::*;

fn main() {
    mount_to_body(|| {
        view! {
            hi
        }
    })
}

#[component]
fn TitleBar(#[prop(into)] title: MaybeSignal<String>) -> impl IntoView {
    view! {}
}

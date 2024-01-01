use leptos::*;

#[component]
pub fn Select(value: RwSignal<String>, children: Children) -> impl IntoView {
    let children = children()
        .nodes
        .into_iter()
        .filter(|n| {
            n.clone()
                .into_html_element()
                .is_ok_and(|e| e.tag_name() == "option")
        })
        .map(|o| o);
    view! {}
}

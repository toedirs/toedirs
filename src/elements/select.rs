use leptos::{html::Input, *};
use wasm_bindgen::JsCast;
use web_sys::HtmlOptionElement;

#[component]
pub fn Select(value: RwSignal<String>, children: Children) -> impl IntoView {
    let show_dropdown = create_rw_signal(false);
    let children = children()
        .nodes
        .into_iter()
        // .filter(|n| {
        //     n.clone()
        //         .into_html_element()
        //         .is_ok_and(|e| e.tag_name() == "option")
        // })
        .map(|o| {
            let el = o.clone();
            let el = el.into_html_element().unwrap();
            let el = el.dyn_ref::<HtmlOptionElement>().unwrap();
            view! {
                <li on:click=move |_| {
                    show_dropdown.set(false);
                }>
                    <span inner_html=el.inner_html()></span>
                </li>
            }
        })
        .collect_view();
    let dropdown_ref = create_node_ref::<Input>();
    view! {
        <div class="select-wrapper">
            <input
                node_ref=dropdown_ref
                class="select-dropdown dropdown-trigger"
                type="text"
                readonly="true"
                on:click=move |_| {
                    show_dropdown.set(true);
                }
            />

            <ul
                class="dropdown-content select-dropdown"
                style=move || {
                    if show_dropdown.get() {
                        format!(
                            "display:block;opacity:1;{}px;",
                            dropdown_ref
                                .get()
                                .and_then(|d| Some(d.get_bounding_client_rect().width()))
                                .unwrap_or(350.0),
                        )
                    } else {
                        "".to_string()
                    }
                }
            >

                {children}
            </ul>
            <i class="valign-wrapper material-symbols-rounded caret">expand_more</i>
        </div>
    }
}

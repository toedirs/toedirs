use std::collections::HashMap;

use leptos::{html::Input, *};
use wasm_bindgen::JsCast;
use web_sys::HtmlOptionElement;

#[component]
pub fn Select(value: RwSignal<String>, children: Children) -> impl IntoView {
    let show_dropdown = create_rw_signal(false);
    let display_value = create_rw_signal("");
    let children: Vec<_> = children()
        .nodes
        .iter()
        .map(|n| {
            let el = n.clone();
            let el = el.into_html_element().unwrap().clone();
            let el = el.dyn_ref::<HtmlOptionElement>().unwrap();
            (
                el.value(),
                el.inner_html(),
                el.get_attribute("disabled").is_some(),
            )
        })
        .collect();
    let value_map: HashMap<_, _> = children
        .iter()
        .map(|(val, inner, _)| (val, inner))
        .collect();
    let children = children
        .into_iter()
        .map(|(val, inner, disabled)| {
            let class = format!(
                "{} {}",
                if value() == val { "selected" } else { "" },
                if disabled { "disabled" } else { "" }
            );
            view! {
                <li
                    class=class
                    on:click=move |_| {
                        value.set(val.clone());
                        show_dropdown.set(false);
                    }
                >

                    <span inner_html=inner.clone()></span>
                </li>
            }
        })
        .collect_view();
    let dropdown_ref = create_node_ref::<Input>();
    view! {
        <div class="select-wrapper">
            <div class="select-dropdown dropdown-trigger">{display_value}</div>
            <input
                node_ref=dropdown_ref
                class="select-dropdown dropdown-trigger"
                style="display:none"
                type="text"
                readonly="true"
                on:click=move |_| {
                    show_dropdown.set(true);
                }

                value=value
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

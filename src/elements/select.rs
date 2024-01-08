use std::collections::HashMap;

use leptos::{html::Input, *};
use wasm_bindgen::JsCast;
use web_sys::HtmlOptionElement;

#[component]
pub fn Select(
    name: &'static str,
    value: RwSignal<String>,
    options: Option<Vec<(String, String, bool)>>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    children: Children,
) -> impl IntoView {
    let show_dropdown = create_rw_signal(false);
    let display_value = create_rw_signal("".to_string());
    let children: Vec<_> = if let Some(children) = options {
        children
    } else {
        children()
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
            .collect()
    };
    let children_copy = children.clone();
    let value_map: HashMap<_, _> = children_copy
        .iter()
        .map(|(val, inner, _)| (val.clone(), inner.clone()))
        .collect();
    let children = children
        .into_iter()
        .map(|(val, inner, disabled)| {
            let class = format!(
                "{} {}",
                if value() == val { "selected" } else { "" },
                if disabled { "disabled" } else { "" }
            );
            if value() == val {
                value_map
                    .clone()
                    .get(&val)
                    .and_then(|v| Some(display_value.set(v.clone())));
            }
            let valmap = value_map.clone();
            view! {
                <li
                    class=class
                    on:click=move |_| {
                        if disabled {
                            return;
                        }
                        value.set(val.clone());
                        valmap.get(&val).and_then(|v| Some(display_value.set(v.clone())));
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
            <div
                class="select-dropdown dropdown-trigger input-field valign-wrapper"
                on:click=move |_| {
                    show_dropdown.set(true);
                }

                {..attrs.clone()}
                inner_html=display_value
            ></div>
            <input
                name=name
                node_ref=dropdown_ref
                class="select-dropdown dropdown-trigger"
                style="display:none"
                type="text"
                readonly="true"
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
            <i
                class="valign-wrapper material-symbols-rounded caret dropdown-trigger"
                style="top:auto;"
                on:click=move |_| {
                    show_dropdown.set(true);
                }
            >

                expand_more
            </i>
        </div>
    }
}

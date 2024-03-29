#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use itertools::Itertools;
use leptos::{ev::SubmitEvent, html::Label, logging::log, *};
use leptos_router::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
use wasm_bindgen::JsCast;
use web_sys::{DragEvent, HtmlElement};

use crate::{elements::select::Select, workout_schedule::WorkoutType};

#[derive(Clone, Debug)]
pub struct Parameter {
    key: u32,
    name: RwSignal<String>,
    value: RwSignal<u32>,
    param_type: RwSignal<String>,
    scaling: RwSignal<bool>,
    order: RwSignal<u32>,
}

impl Default for Parameter {
    fn default() -> Self {
        Self {
            key: 0,
            name: create_rw_signal("".to_string()),
            value: create_rw_signal(1),
            param_type: create_rw_signal("time_s".to_string()),
            scaling: create_rw_signal(false),
            order: create_rw_signal(0),
        }
    }
}
#[component]
pub fn WorkoutParameter(param: Parameter) -> impl IntoView {
    let select_name = format!("param[{}][param_type]", param.key).to_string();
    view! {
        <div class="workout-parameter" draggable="true">
            <div class="columns">
                <input
                    type="hidden"
                    name=move || format!("param[{}][position]", param.key)
                    value=param.order
                />

                <div class="column is-fullwidth">
                    <div class="box">
                        <div class="field">
                            <label class="label is-small" for=move || format!("name-{}", param.key)>
                                <span class="icon-text">
                                    <span class="icon has-text-black">
                                        <i class="fas fa-grip-vertical"></i>
                                    </span>
                                    Name
                                </span>
                            </label>
                            <div class="control">
                                <input
                                    class="input is-small"
                                    id=move || format!("name-{}", param.key)
                                    name=move || format!("param[{}][name]", param.key)
                                    type="text"
                                    value=param.name
                                />

                            </div>
                        </div>
                        <div class="field is-grouped">
                            <p class="control">
                                <input
                                    class="input is-small"
                                    type="number"
                                    name=move || format!("param[{}][value]", param.key)
                                    min="1"
                                    value=param.value
                                />

                            </p>
                            <p class="control">
                                <div class="select is-small">
                                    <select
                                        value=param.param_type
                                        name=select_name
                                        id="parameter_type"
                                    >
                                        <option value="time_s">Time</option>
                                        <option value="distance_m">Distance(m)</option>
                                        <option value="trainingload">TrainingLoad</option>
                                    </select>
                                </div>
                            // <Select
                            // value=param.param_type
                            // name=select_name
                            // options=None
                            // attr:id="parameter_type"
                            // >
                            // <option value="time_s">Time</option>
                            // <option value="distance_m">Distance(m)</option>
                            // <option value="trainingload">TrainingLoad</option>
                            // </Select>
                            </p>
                            <p class="control">
                                <input
                                    type="hidden"
                                    name=move || format!("param[{}][scaling]", param.key)
                                    value=move || if param.scaling.get() { "true" } else { "false" }
                                />
                                <label class="radio is-small">
                                    <input
                                        type="checkbox"
                                        checked=param.scaling
                                        on:change=move |_| {
                                            param.scaling.update(|v| *v = !*v);
                                        }
                                    />

                                    Scaling
                                </label>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutParam {
    name: String,
    value: i32,
    param_type: String,
    scaling: bool,
    position: i32,
}

#[server]
pub async fn create_workout(
    name: String,
    workout_type: String,
    param: Vec<WorkoutParam>,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let result = sqlx::query!(
        r#"
        INSERT INTO workout_templates (user_id, template_name, workout_type)
        VALUES ($1, $2,$3)
        RETURNING id
        "#,
        user.id as _,
        name,
        TryInto::<WorkoutType>::try_into(workout_type)
            .map_err(|_| ServerFnError::new("Couldn't parse workout type".to_string()))?
            as _
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Error saving workout template: {}", e)))?;
    let template_ids: Vec<i64> = std::iter::repeat(result.id).take(param.len()).collect();
    let (names, types, values, scalings, positions): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
        param
            .into_iter()
            .map(|p| (p.name, p.param_type, p.value, p.scaling, p.position))
            .multiunzip();
    sqlx::query!(
        r#"
        INSERT INTO workout_parameters(workout_template_id,name,parameter_type,value,scaling,position)
        SELECT *
        FROM UNNEST($1::bigint[], $2::text[], $3::workout_parameter_type[], $4::integer[], $5::boolean[], $6::integer[])
        "#,
        &template_ids[..],
        &names[..],
        &types[..] as _,
        &values[..] as _,
        &scalings[..],
        &positions
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't insert workout parameters: {}", e)))?;

    Ok(())
}

#[component]
pub fn CreateWorkoutDialog(show: RwSignal<bool>) -> impl IntoView {
    let select_value = create_rw_signal("".to_string());
    let create_workout_action = create_server_action::<CreateWorkout>();
    let workout_parameter_index = create_rw_signal(0);
    let workout_parameters = create_rw_signal(vec![Parameter::default()]);
    let on_submit = move |ev: SubmitEvent| {
        log!("{:?}", ev);
        let data = CreateWorkout::from_event(&ev);
        log!("{:?}", data);
        show.set(false);
    };
    let owner = Owner::current().unwrap();
    let _ = watch(
        move || show.get(),
        move |cur, prev, _| {
            if *cur && !*prev.unwrap_or(&false) {
                workout_parameters.set(Vec::new());
                workout_parameter_index.set(0);
            }
        },
        false,
    );
    let _ = watch(
        move || workout_parameter_index.get(),
        move |num, _, _| {
            workout_parameters.update(|v| {
                with_owner(owner, move || {
                    v.push(Parameter {
                        key: *num,
                        order: create_rw_signal(*num),
                        ..Default::default()
                    });
                });
            });
        },
        false,
    );
    let name_ref = create_node_ref::<Label>();
    let close = move |_| show.set(false);
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <ActionForm action=create_workout_action on:submit=on_submit>
                <div class="modal is-active">
                    <div class="modal-background" on:click=close></div>
                    <div class="modal-card">
                        <div class="modal-card-head">
                            <p class="modal-card-title">"Create workout"</p>
                            <button class="delete" aria-label="close" on:click=close></button>

                        </div>
                        <div class="modal-card-body">
                            <div class="field">
                                <label class="label" for="name">
                                    Name
                                </label>
                                <div class="control">
                                    <input class="input" id="name" name="name" type="text"/>

                                </div>
                            </div>
                            <div class="field">
                                <label class="label" for="workout_type">
                                    Type
                                </label>
                                <div class="control">
                                    <div class="select">
                                        <select name="workout_type">
                                            <option value="" disabled selected>
                                                Choose Workout Type
                                            </option>
                                            <option value="run">Run</option>
                                            <option value="strength">Strength</option>
                                            <option value="cycling">Cycling</option>
                                            <option value="hiking">Hiking</option>
                                            <option value="endurance">General Endurance</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                            <div class="columns">
                                <div class="column is-full-width">
                                    <For each=workout_parameters key=|s| s.key let:child>
                                        <WorkoutParameter
                                            param=child.clone()
                                            on:dragstart=move |ev: DragEvent| {
                                                let dt = ev.data_transfer().unwrap();
                                                dt.set_data("key", child.key.to_string().as_str()).unwrap();
                                            }

                                            on:dragover=move |ev: DragEvent| {
                                                ev.prevent_default();
                                                let dt = ev.data_transfer().unwrap();
                                                let drag_key = dt
                                                    .get_data("key")
                                                    .unwrap()
                                                    .parse::<u32>()
                                                    .unwrap();
                                                if drag_key == child.key {
                                                    return;
                                                }
                                                let src_index = workout_parameters
                                                    .get_untracked()
                                                    .iter()
                                                    .position(|e| e.key == drag_key)
                                                    .unwrap();
                                                let tgt_index = workout_parameters
                                                    .get_untracked()
                                                    .iter()
                                                    .position(|e| e.key == child.key)
                                                    .unwrap();
                                                let tgt = ev.target().unwrap();
                                                let parent = tgt
                                                    .dyn_ref::<HtmlElement>()
                                                    .unwrap()
                                                    .closest(".workout-parameter")
                                                    .unwrap()
                                                    .unwrap();
                                                let cls_name = parent.class_name();
                                                let mut cls = cls_name.split(" ").collect::<Vec<_>>();
                                                if !cls.contains(&"drag-over") {
                                                    cls.push("drag-over");
                                                    if src_index > tgt_index {
                                                        cls.push("before");
                                                    } else {
                                                        cls.push("after");
                                                    }
                                                    let cls_name = cls.join(" ");
                                                    parent.set_class_name(cls_name.as_str());
                                                }
                                            }

                                            on:dragleave=move |ev: DragEvent| {
                                                let tgt = ev.target().unwrap();
                                                let parent = tgt
                                                    .dyn_ref::<HtmlElement>()
                                                    .unwrap()
                                                    .closest(".workout-parameter")
                                                    .unwrap()
                                                    .unwrap();
                                                let cls_name = parent.class_name();
                                                let cls = cls_name.split(" ").collect::<Vec<_>>();
                                                if cls.contains(&"drag-over") {
                                                    let cls_name = cls
                                                        .iter()
                                                        .filter(|&v| {
                                                            v != &"drag-over" && v != &"after" && v != &"before"
                                                        })
                                                        .map(|v| *v)
                                                        .collect::<Vec<_>>()
                                                        .join(" ");
                                                    parent.set_class_name(cls_name.as_str());
                                                }
                                            }

                                            on:drop=move |ev: DragEvent| {
                                                ev.prevent_default();
                                                let dt = ev.data_transfer().unwrap();
                                                let drag_key = dt
                                                    .get_data("key")
                                                    .unwrap()
                                                    .parse::<u32>()
                                                    .unwrap();
                                                let tgt = ev.target().unwrap();
                                                let parent = tgt
                                                    .dyn_ref::<HtmlElement>()
                                                    .unwrap()
                                                    .closest(".workout-parameter")
                                                    .unwrap()
                                                    .unwrap();
                                                let cls_name = parent.class_name();
                                                let cls = cls_name.split(" ").collect::<Vec<_>>();
                                                if cls.contains(&"drag-over") {
                                                    let cls_name = cls
                                                        .iter()
                                                        .filter(|&v| {
                                                            v != &"drag-over" && v != &"before" && v != &"after"
                                                        })
                                                        .map(|v| *v)
                                                        .collect::<Vec<_>>()
                                                        .join(" ");
                                                    parent.set_class_name(cls_name.as_str());
                                                }
                                                if child.key == drag_key {
                                                    return;
                                                }
                                                workout_parameters
                                                    .update(|v| {
                                                        let src_index = v
                                                            .iter()
                                                            .position(|e| e.key == drag_key)
                                                            .unwrap();
                                                        let tgt_index = v
                                                            .iter()
                                                            .position(|e| e.key == child.key)
                                                            .unwrap();
                                                        let el = v.remove(src_index);
                                                        v.insert(tgt_index, el);
                                                        for i in 0..v.len() {
                                                            v[i].order.set(i as u32);
                                                        }
                                                    });
                                            }
                                        />

                                    </For>
                                </div>
                            </div>
                            <div class="columns is-centered">
                                <div class="column is-narrow">
                                    <a
                                        class="button"
                                        on:click=move |_| {
                                            workout_parameter_index.update(|v| *v = *v + 1)
                                        }
                                    >

                                        <i class="large material-symbols-rounded">add</i>
                                    </a>
                                </div>
                            </div>
                        </div>
                        <div class="modal-card-foot">
                            <button class="button" on:click=close>
                                Cancel
                            </button>
                            <button type="submit" class="button is-success">
                                <i class="material-symbols-rounded right">save</i>
                                Create
                            </button>
                        </div>
                    </div>
                </div>
            </ActionForm>
            <div
                class="modal-overlay"
                style="z-index: 1002; display: block; opacity: 0.5;"
                on:click=move |_| { show.set(false) }
            ></div>
        </Show>
    }
}

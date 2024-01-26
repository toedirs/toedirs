#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use leptos::{ev::SubmitEvent, logging::log, *};
use leptos_router::*;
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
use strum;
use thaw::*;
use wasm_bindgen::JsCast;
use web_sys::{DragEvent, HtmlElement};

use crate::{elements::select::Select, workout_schedule::WorkoutType};

#[derive(Clone)]
pub struct Parameter {
    key: u32,
    name: RwSignal<String>,
    value: RwSignal<u32>,
    param_type: RwSignal<String>,
    scaling: RwSignal<bool>,
}

impl Default for Parameter {
    fn default() -> Self {
        Self {
            key: 0,
            name: create_rw_signal("".to_string()),
            value: create_rw_signal(1),
            param_type: create_rw_signal("time".to_string()),
            scaling: create_rw_signal(false),
        }
    }
}
#[component]
pub fn WorkoutParameter(param: Parameter) -> impl IntoView {
    let select_name = format!("param[{}][param_type]", param.key).to_string();
    view! {
        <div class="row workout-parameter" draggable="true">

            <div class="col s12">
                <div class="row">
                    <div class="col s12">
                        <label for=move || format!("name-{}", param.key)>Name</label>
                        <input
                            id=move || format!("name-{}", param.key)
                            name=move || format!("param[{}][name]", param.key)
                            type="text"
                            value=param.name
                        />
                    </div>
                </div>
                <div class="row">
                    <div class="col s4">
                        <input
                            type="number"
                            name=move || format!("param[{}][value]", param.key)
                            min="1"
                            value=param.value
                        />

                    </div>
                    <div class="col s4">
                        <Select
                            value=param.param_type
                            name=select_name
                            options=None
                            attr:id="parameter_type"
                        >
                            <option value="time">Time</option>
                            <option value="distance">Distance(m)</option>
                            <option value="trainingload">TrainingLoad</option>
                        </Select>
                    </div>
                    <div class="col s4 switch">
                        <label>
                            Scaling
                            <input
                                type="checkbox"
                                name=move || format!("param[{}][scaling]", param.key)
                                checked=param.scaling
                                on:change=move |_| {
                                    param.scaling.update(|v| *v = !*v);
                                }
                            />
                            <span class="lever"></span>
                        </label>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[server(CreateWorkout, "/api")]
pub async fn create_workout(name: String, workout_type: String) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::ServerError("Not logged in".to_string()))?;
    sqlx::query!(
        r#"
        INSERT INTO workout_templates (user_id, template_name, workout_type)
        VALUES ($1, $2,$3)
        "#,
        user.id as _,
        name,
        TryInto::<WorkoutType>::try_into(workout_type)
            .map_err(|_| ServerFnError::ServerError("Couldn't parse workout type".to_string()))?
            as _
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(format!("Error saving workout template: {}", e)))?;
    Ok(())
}

#[component]
pub fn CreateWorkoutDialog(show: RwSignal<bool>) -> impl IntoView {
    let on_submit = move |_ev: SubmitEvent| {
        show.set(false);
    };
    let select_value = create_rw_signal("".to_string());
    let create_workout_action = create_server_action::<CreateWorkout>();
    let workout_parameter_index = create_rw_signal(0);
    let workout_parameters = create_rw_signal(vec![Parameter::default()]);
    watch(
        move || show.get(),
        move |cur, prev, _| {
            if *cur && !*prev.unwrap_or(&false) {
                workout_parameters.set(Vec::new());
                workout_parameter_index.set(0);
            }
        },
        false,
    );
    watch(
        move || workout_parameter_index.get(),
        move |num, _, _| {
            workout_parameters.update(|v| {
                v.push(Parameter {
                    key: *num,
                    ..Default::default()
                })
            });
        },
        false,
    );
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <ActionForm action=create_workout_action on:submit=on_submit>
                <div class="modal" style="z-index: 1003; ">
                    <div class="modal-header">
                        <h4 class="black-text">"Create workout"</h4>

                    </div>
                    <div class="modal-body">
                        <div class="modal-content">
                            <div class="row">
                                <div class="col s6 input-field">
                                    <input id="name" name="name" type="text"/>
                                    <label for="name">Name</label>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col s6 input-field">
                                    <Select
                                        value=select_value
                                        name="workout_type".to_string()
                                        options=None
                                        attr:id="workout_type"
                                    >
                                        <option value="" disabled selected>
                                            Choose workout type
                                        </option>
                                        <option value="run">
                                            <i class="material-symbols-rounded">directions_run</i>
                                            Run
                                        </option>
                                        <option value="strength">
                                            <i class="material-symbols-rounded">fitness_center</i>
                                            Strength
                                        </option>
                                        <option value="cycling">
                                            <i class="material-symbols-rounded">directions_bike</i>
                                            Cycling
                                        </option>
                                        <option value="hiking">
                                            <i class="material-symbols-rounded">directions_walk</i>
                                            Hike
                                        </option>
                                        <option value="endurance">
                                            <i class="material-symbols-rounded">directions_walk</i>
                                            General Endurance
                                        </option>
                                    </Select>
                                    <label for="workout_type">Type</label>
                                </div>
                                <div class="row">
                                    <div class="col s12">
                                        <For each=workout_parameters key=|s| s.key let:child>
                                            <WorkoutParameter
                                                param=child.clone()
                                                on:dragstart=move |ev: DragEvent| {
                                                    let dt = ev.data_transfer().unwrap();
                                                    dt.set_data("key", child.key.to_string().as_str()).unwrap();
                                                }

                                                on:dragover=move |ev: DragEvent| {
                                                    ev.prevent_default();
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
                                                            .filter(|&v| v != &"drag-over")
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
                                                            .filter(|&v| v != &"drag-over")
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
                                                        });
                                                }
                                            />

                                        </For>
                                        <div class="row">
                                            <div class="col s2 offset-s5 center-align">
                                                <a
                                                    class="btn-floating btn-large teal"
                                                    on:click=move |_| {
                                                        workout_parameter_index.update(|v| *v = *v + 1)
                                                    }
                                                >

                                                    <i class="large material-symbols-rounded">add</i>
                                                </a>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="submit" class="btn waves-effect waves-light">
                            <i class="material-symbols-rounded right">save</i>
                            Create
                        </button>
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

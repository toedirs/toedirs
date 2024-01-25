#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use leptos::{ev::SubmitEvent, *};
use leptos_router::*;
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
use strum;
use thaw::*;

use crate::{elements::select::Select, workout_schedule::WorkoutType};

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
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <div
                class="modal"
                style="z-index: 1003; display: block; opacity: 1; top: 10%;overflow:visible;"
            >
                <ActionForm action=create_workout_action on:submit=on_submit>
                    <div class="modal-content" style="overflow:visible;">
                        <h4 class="black-text">"Create workout"</h4>
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
                                    name="workout_type"
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
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="submit" class="btn waves-effect waves-light">
                            <i class="material-symbols-rounded right">save</i>
                            Create
                        </button>
                    </div>
                </ActionForm>
            </div>
            <div
                class="modal-overlay"
                style="z-index: 1002; display: block; opacity: 0.5;"
                on:click=move |_| { show.set(false) }
            ></div>
        </Show>
    }
}

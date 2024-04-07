#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
#[cfg(feature = "ssr")]
use crate::models::get_user_preferences;
use crate::models::UserPreferences;
#[cfg(feature = "ssr")]
use chrono::Local;
#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};
use leptos::*;
use leptos_router::*;
#[cfg(feature = "ssr")]
use sqlx::*;

#[cfg(feature = "ssr")]
use nalgebra::DVector;
#[cfg(feature = "ssr")]
use varpro::{
    prelude::*,
    solvers::levmar::{LevMarProblemBuilder, LevMarSolver},
};

#[cfg(feature = "ssr")]
fn exp_model(x: &DVector<f64>, tau: f64) -> DVector<f64> {
    x.map(|x| (tau * x).exp() + 1.0)
}
#[cfg(feature = "ssr")]
fn exp_model_dtau(tvec: &DVector<f64>, tau: f64) -> DVector<f64> {
    tvec.map(|t| t * (tau * t).exp())
}

#[cfg(feature = "ssr")]
fn curve_fit(aerobic: f64, anaerobic: f64, max_heartrate: f64) -> (f64, f64) {
    let x = DVector::from_vec(vec![aerobic, anaerobic, (anaerobic + max_heartrate) / 2.0]);
    let y = DVector::from_vec(vec![0.71, 2.61, 4.16]);
    let model = SeparableModelBuilder::<f64>::new(&["tau"])
        .function(&["tau"], exp_model)
        .partial_deriv("tau", exp_model_dtau)
        // .invariant_function(|x| DVector::from_element(x.len(), 1.0))
        .independent_variable(x)
        .initial_parameters(vec![0.01])
        .build()
        .unwrap();
    let problem = LevMarProblemBuilder::new(model)
        .observations(y)
        .build()
        .unwrap();
    let fit_result = LevMarSolver::default()
        .fit(problem)
        .expect("fit must succeed");
    let alpha = fit_result.nonlinear_parameters();
    let c = fit_result.linear_coefficients().unwrap();
    (alpha[0], c[0])
}

#[server]
pub async fn update_user_preferences(
    aerobic_threshold: u32,
    anaerobic_threshold: u32,
    max_heartrate: u32,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let (tau, c) = curve_fit(
        aerobic_threshold as f64,
        anaerobic_threshold as f64,
        max_heartrate as f64,
    );
    let current = sqlx::query!(
        r#"
        SELECT id
        FROM user_preferences
        WHERE user_id=$1 AND end_time is NULL
        "#,
        user.id as _
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't query user preferences:{}", e)))?;
    match current {
        Some(current) => {
            let mut transaction = pool
                .begin()
                .await
                .map_err(|e| ServerFnError::new(format!("Couldn't start transaction:{}", e)))?;
            sqlx::query!(
                r#"
                INSERT INTO user_preferences (user_id,start_time,end_time,aerobic_threshold, anaerobic_threshold,max_heartrate, tau, c)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                "#,
                user.id as _,
                Utc::now(),
                Option::<DateTime<Utc>>::None,
                aerobic_threshold as i32,
                anaerobic_threshold as i32,
                max_heartrate as i32,
                tau,
                c
            ).execute(&mut *transaction).await.map_err(|e|ServerFnError::new(format!("Couldn't update preferences:{}",e)))?;

            sqlx::query!(
                r#"
                    UPDATE user_preferences
                    SET end_time=$2
                    WHERE id=$1
                "#,
                current.id,
                Utc::now()
            )
            .execute(&mut *transaction)
            .await
            .map_err(|e| ServerFnError::new(format!("Couldn't update preferences:{}", e)))?;
            transaction
                .commit()
                .await
                .map_err(|e| ServerFnError::new(format!("Couldn't commit changes:{}", e)))?;
        }
        None => {
            sqlx::query!(
                r#"
                INSERT INTO user_preferences (user_id,start_time,end_time,aerobic_threshold, anaerobic_threshold,max_heartrate, tau, c)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                "#,
                user.id as _,
                Option::<DateTime<Utc>>::None,
                Option::<DateTime<Utc>>::None,
                aerobic_threshold as i32,
                anaerobic_threshold as i32,
                max_heartrate as i32,
                tau,
                c
            ).execute(&pool).await.map_err(|e|ServerFnError::new(format!("Couldn't update preferences:{}",e)))?;
        }
    }
    Ok(())
}

#[server]
pub async fn get_preferences() -> Result<UserPreferences, ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let preferences = get_user_preferences(user.id, Local::now(), &pool).await;
    Ok(preferences)
}

#[component]
pub fn UserSettings(show: RwSignal<bool>) -> impl IntoView {
    let close = move |_| show.set(false);
    let aerobic_threshold = create_rw_signal(140);
    let anaerobic_threshold = create_rw_signal(160);
    let max_heartrate = create_rw_signal(180);
    let update_user_preferences = create_server_action::<UpdateUserPreferences>();
    spawn_local(async move {
        let preferences = get_preferences()
            .await
            .expect("can't load user preferences");
        aerobic_threshold.set(preferences.aerobic_threshold as u32);
        anaerobic_threshold.set(preferences.anaerobic_threshold as u32);
        max_heartrate.set(preferences.max_heartrate as u32);
    });
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <ActionForm
                action=update_user_preferences
                on:submit=move |_| {
                    show.set(false);
                }
            >

                <div class="modal is-active">
                    <div class="modal-background" on:click=close></div>
                    <div class="modal-card">
                        <div class="modal-card-head">
                            <p class="modal-card-title">"User Settings"</p>
                            <button class="delete" aria-label="close" on:click=close></button>
                        </div>
                        <div class="modal-card-body">
                            <div class="columns">
                                <div class="column is-full">
                                    <div class="field">
                                        <label class="label">Aerobic Threshold</label>
                                        <div class="control">
                                            <div class="field has-addons">
                                                <div class="control is-expanded">
                                                    <input
                                                        class="input"
                                                        type="range"
                                                        name="aerobic_threshold"
                                                        min="60"
                                                        max=anaerobic_threshold
                                                        step="1"
                                                        value=aerobic_threshold
                                                        on:input=move |ev| {
                                                            let value = event_target_value(&ev).parse::<u32>();
                                                            if let Ok(value) = value {
                                                                aerobic_threshold.set(value);
                                                            }
                                                        }
                                                    />

                                                </div>
                                                <div class="control">
                                                    <span class="tag is-medium is-info ml-2">
                                                        {move || aerobic_threshold()}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="columns">
                                <div class="column is-full">
                                    <div class="field">
                                        <label class="label">Anaerobic Threshold</label>
                                        <div class="control">
                                            <div class="field has-addons">
                                                <div class="control is-expanded">
                                                    <input
                                                        class="input"
                                                        type="range"
                                                        name="anaerobic_threshold"
                                                        min=aerobic_threshold
                                                        max=max_heartrate
                                                        step="1"
                                                        value=anaerobic_threshold
                                                        on:input=move |ev| {
                                                            let value = event_target_value(&ev).parse::<u32>();
                                                            if let Ok(value) = value {
                                                                anaerobic_threshold.set(value);
                                                            }
                                                        }
                                                    />

                                                </div>
                                                <div class="control">
                                                    <span class="tag is-medium is-warning ml-2">
                                                        {move || anaerobic_threshold()}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="columns">
                                <div class="column is-full">
                                    <div class="field">
                                        <label class="label">Max Heartrate</label>
                                        <div class="control">
                                            <div class="field has-addons">
                                                <div class="control is-expanded">
                                                    <input
                                                        class="input"
                                                        type="range"
                                                        name="max_heartrate"
                                                        min=anaerobic_threshold
                                                        max="220"
                                                        step="1"
                                                        value=max_heartrate
                                                        on:input=move |ev| {
                                                            let value = event_target_value(&ev).parse::<u32>();
                                                            if let Ok(value) = value {
                                                                max_heartrate.set(value);
                                                            }
                                                        }
                                                    />

                                                </div>
                                                <div class="control">
                                                    <span class="tag is-medium is-danger ml-2">
                                                        {move || max_heartrate()}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                        </div>
                        <div class="modal-card-foot">
                            <button class="button" on:click=close>
                                Cancel
                            </button>
                            <button type="submit" class="button is-success">
                                <i class="material-symbols-rounded right">save</i>
                                Save
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "ssr")]
    use super::curve_fit;

    #[cfg(feature = "ssr")]
    #[test]
    fn test_hr_fit() {
        let fit = curve_fit(155.0, 172.0, 183.0);
        println!("{:?}", fit);
        assert!((fit.0 - 0.0809749).abs() < 0.00001);
        assert!((fit.1 - 0.000002370473).abs() < 0.00000000001);
    }
}

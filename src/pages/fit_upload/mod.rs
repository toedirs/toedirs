use std::time::Duration;

#[cfg(feature = "ssr")]
use anyhow::{bail, Context, Result};
#[cfg(feature = "ssr")]
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
#[cfg(feature = "ssr")]
use bytes::Bytes;
use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_router::*;
#[cfg(feature = "ssr")]
use sqlx::PgPool;

use crate::app::FitFileUploaded;

#[cfg(feature = "ssr")]
use crate::auth::User;
#[cfg(feature = "ssr")]
use crate::models::{
    activity::{insert_activity, Activity},
    base::{DatabaseEntry, New},
    lap::{insert_laps, Lap},
    record::{insert_records, Record},
    session::{insert_sessions, Session},
};
#[cfg(feature = "ssr")]
use crate::state::AppState;
#[cfg(feature = "ssr")]
use axum_session_auth::{AuthSession, SessionPgPool};

#[cfg(feature = "ssr")]
pub async fn upload_fit_file(
    State(state): State<AppState>,
    auth: AuthSession<User, i64, SessionPgPool, PgPool>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user = if let Some(user) = auth.current_user {
        user
    } else {
        return (StatusCode::FORBIDDEN, "Not logged in".to_string());
    };
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();
        let result = process_fit_file(data, user.id, state.pool.clone()).await;
        if let Err(x) = result {
            return (StatusCode::BAD_REQUEST, format!("{}", x));
        }
    }
    (StatusCode::ACCEPTED, "".to_string())
}

#[cfg(feature = "ssr")]
async fn process_fit_file<'a>(data: Bytes, user_id: i64, executor: PgPool) -> Result<()> {
    use crate::models::user_preferences::get_user_preferences;

    let mut records: Vec<DatabaseEntry<New, Record>> = Vec::new();
    let mut sessions: Vec<DatabaseEntry<New, Session>> = Vec::new();
    let mut laps: Vec<DatabaseEntry<New, Lap>> = Vec::new();
    let mut activity: Option<DatabaseEntry<New, Activity>> = None;
    for data in fitparser::from_bytes(&data).context("Failed to read fit file")? {
        match data.kind() {
            fitparser::profile::MesgNum::Record => {
                DatabaseEntry::<New, Record>::try_from(data)
                    .map(|record| records.push(record))
                    .context("Couldn't parse record")?;
            }
            fitparser::profile::MesgNum::Event => {
                leptos::logging::log!("Event: {:?}", data);
            }
            fitparser::profile::MesgNum::Session => {
                DatabaseEntry::<New, Session>::try_from(data)
                    .map(|session| sessions.push(session))
                    .context("Couldn't parse record")?;
            }
            fitparser::profile::MesgNum::Lap => {
                DatabaseEntry::<New, Lap>::try_from(data)
                    .map(|lap| laps.push(lap))
                    .context("Couldn't parse record")?;
            }
            fitparser::profile::MesgNum::Activity => {
                if activity.is_some() {
                    bail!("Found more than one activity");
                }
                activity = Some(
                    DatabaseEntry::<New, Activity>::try_from(data)
                        .expect("no activity entry found"),
                );
            }
            fitparser::profile::MesgNum::DeviceInfo => {
                leptos::logging::log!("Device Info: {:?}", data);
            }
            _ => {
                leptos::logging::log!("Unknown: {:?}", data.kind())
            }
        }
    }
    if let Some(mut activity) = activity {
        let hr_measurements: Vec<_> = records
            .iter()
            .filter_map(|r| r.state.heartrate.and_then(|hr| Some(hr as u32)))
            .collect();
        if hr_measurements.len() > 0 {
            activity.state.avg_heartrate =
                Some((hr_measurements.iter().sum::<u32>() / hr_measurements.len() as u32) as u16);
            // calculate training load
            let preferences =
                get_user_preferences(user_id, activity.state.start_time, &executor).await;
            activity.state.load = Some(preferences.calculate_load(hr_measurements));
        }

        let mut tx = executor.begin().await?;
        let result = insert_activity(activity, user_id, &mut *tx).await;
        if let Err(x) = result {
            bail!("activity wasn't inserted: {}", x);
        };
        let activity = result.unwrap();
        let result = insert_records(records, activity.extra.activity_id, &mut *tx).await;
        if let Err(x) = result {
            bail!("couldn't insert records: {}", x);
        }
        let result = insert_sessions(sessions, activity.extra.activity_id, &mut *tx).await;
        if let Err(x) = result {
            bail!("couldn't insert sessions: {}", x);
        }
        let result = insert_laps(laps, activity.extra.activity_id, &mut *tx).await;
        if let Err(x) = result {
            bail!("couldn't insert laps: {}", x);
        }
        let tx_result = tx.commit().await;
        if let Err(x) = tx_result {
            bail!("Transaction failed, try again: {}", x);
        };
    } else {
        bail!("No activity found in fit file, may be corrupt");
    };
    Ok(())
}

#[component]
pub fn FitUploadForm(show: ReadSignal<bool>, show_set: WriteSignal<bool>) -> impl IntoView {
    let uploaded = use_context::<FitFileUploaded>().unwrap();
    let on_submit = move |_ev: SubmitEvent| {
        set_timeout(
            move || uploaded.0.update(|v| *v += 1),
            Duration::from_secs(2),
        );
        show_set(false);
    };
    let close = move |_| show_set(false);
    leptos::view! {
        <Show when=move || { show() } fallback=|| { }>
            <Form
                action="/api/upload_fit_file"
                method="POST"
                enctype="multipart/form-data".to_string()
                on:submit=on_submit
            >
                <div
                    class="modal is-active"
                >
                <div class="modal-background" on:click=close></div>
                <div class="modal-card">
                    <div class="modal-card-head">
                            <p class="modal-card-title">"Upload Fit File"</p>
                            <button class="delete" aria-label="close" on:click=close></button>
                    </div>
                        <div class="modal-card-body">
                            <div class="file">
                                <label class="file-label">
                                    <input class="file-input" type="file" name="fit_file" multiple />
                                    <span class="file-cta">
                                        <span class="file-icon">
                                            <i class="fas fa-upload"></i>
                                        </span>
                                        <span class="file-label">
                                            Choose Fit Files...
                                        </span>
                                    </span>
                                </label>
                            </div>
                        </div>
                        <div class="modal-card-foot">
                            <button type="submit" class="button is-success">
                                <i class="material-symbols-rounded right">upload</i>
                                Upload
                            </button>
                            <button class="button" on:click=close>Cancel</button>
                        </div>
                    </div>
                </div>
            </Form>
        </Show>
    }
}

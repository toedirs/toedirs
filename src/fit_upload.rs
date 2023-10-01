use anyhow::{bail, Context, Result};
#[cfg(feature = "ssr")]
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use bytes::Bytes;
use chrono::{DateTime, Local};
use fitparser::{profile::MesgNum, FitDataRecord, Value};
use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_router::*;
#[cfg(feature = "ssr")]
use sqlx::{PgExecutor, PgPool};

use super::auth::User;
#[cfg(feature = "ssr")]
use super::models::{insert_activity, Activity, DatabaseEntry, Lap, New, Record, Session};
#[cfg(feature = "ssr")]
use super::state::AppState;
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
        let result = process_fit_file(data, user.id, &state.pool).await;
        if let Err(x) = result {
            return (StatusCode::BAD_REQUEST, format!("{}", x));
        }
    }
    (StatusCode::ACCEPTED, "".to_string())
}

#[cfg(feature = "ssr")]
async fn process_fit_file<'a>(
    data: Bytes,
    user_id: i64,
    executor: impl PgExecutor<'a>,
) -> Result<()> {
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
                if let Some(_) = activity {
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
    if let Some(activity) = activity {
        let result = insert_activity(activity, user_id, executor).await;
        if let Err(x) = result {
            bail!("activity wasn't inserted: {}", x);
        }
    } else {
        bail!("No activity found in fit file, may be corrupt");
    }
    // leptos::logging::log!("Parsed records: {:?}", records);
    Ok(())
}

#[component]
pub fn FitUploadForm(show: ReadSignal<bool>, show_set: WriteSignal<bool>) -> impl IntoView {
    let on_submit = move |_ev: SubmitEvent| {
        show_set(false);
        println!("hidden?")
    };
    leptos::view! {
        <Show when=move || { show() } fallback=|| { () }>
            <div
                class="modal bottom-sheet"
                style="z-index: 1003; display: block; opacity: 1; bottom: 0%"
            >
                <Form
                    action="/api/upload_fit_file"
                    method="POST"
                    enctype="multipart/form-data".to_string()
                    on:submit=on_submit
                >
                    <div
                        class="modal-content">
                        <h4 class="black-text">"Upload Fit File"</h4>
                        <div class="row">
                            <div class="col s12 input-field file-field">
                                <div class="btn">
                                    <span>File</span>
                                    <input type="file" name="fit_file" multiple/>
                                </div>
                                <div class="file-path-wrapper">
                                    <input class="file-path validate" type="text"/>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="submit" class="btn waves-effect waves-light">
                            <i class="material-symbols-rounded right">upload</i>
                            Upload
                        </button>
                    </div>
                </Form>
            </div>
            <div class="modal-overlay" style="z-index: 1002; display: block; opacity: 0.5;" on:click=move |_|{show_set(false)}></div>
        </Show>
    }
}

use std::time::Duration;

#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::{activity_overview::activity_details::ActivityDetails, error_template::ErrorTemplate};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Local};
use humantime::format_duration;
use leptos::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::*;

pub mod activity_details;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityListEntry {
    pub id: i64,
    pub start_time: DateTime<Local>,
    pub duration: BigDecimal,
    pub sport: String,
}

#[server(ActivityList, "/api")]
pub async fn get_activity_list() -> Result<Vec<ActivityListEntry>, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::new("Not logged in".to_string()));
    }
    let user = auth.current_user.unwrap();
    let pool = pool()?;
    let activities = query_as!(
        ActivityListEntry,
        r#"
        SELECT 
            activities.id, 
            activities.start_time, 
            activities.duration,
            COALESCE(string_agg(sessions.sport,', '),'General') as "sport!" 
        FROM activities 
        JOIN sessions on sessions.activity_id=activities.id
        WHERE activities.user_id = $1::bigint
        GROUP BY activities.id
        ORDER BY activities.start_time DESC"#,
        user.id
    )
    .fetch_all(&pool)
    .await?;
    Ok(activities)
}
#[server]
pub async fn delete_activity(activity_id: i64) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    sqlx::query!(
        r#"
        DELETE FROM activities
        WHERE user_id=$1 and id=$2
        "#,
        user.id as _,
        activity_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't delete activity:{}", e)))?;

    Ok(())
}

#[component]
pub fn ActivityList() -> impl IntoView {
    let activities = create_resource(move || (), move |_| get_activity_list());
    let show_activity = create_rw_signal(None);
    let delete_activity = create_server_action::<DeleteActivity>();
    view! {
        <div class="container">
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors/> }
                }>
                    {move || {
                        activities
                            .get()
                            .map(move |activities| match activities {
                                Err(e) => {
                                    view! { <pre class="error">"Error: " {e.to_string()}</pre> }
                                        .into_view()
                                }
                                Ok(activities) => {
                                    view! {
                                        <For
                                            each=move || activities.clone()
                                            key=|e| e.id
                                            let:activity
                                        >
                                            <div class="columns">
                                                <div class="column is-full level">
                                                    <div class="box level">
                                                        <div class="level-left">
                                                            <div class="block">
                                                                <p class="title is-5">
                                                                    <a
                                                                        href="#!"
                                                                        on:click=move |_| show_activity.set(Some(activity.id))
                                                                    >
                                                                        {activity.sport}
                                                                    </a>
                                                                </p>
                                                                <p class="subtitle is-7">
                                                                    {activity.start_time.format("%Y-%m-%d").to_string()} <br/>
                                                                    {format_duration(
                                                                            Duration::new(activity.duration.to_u64().unwrap(), 0),
                                                                        )
                                                                        .to_string()}
                                                                </p>
                                                            </div>
                                                        </div>
                                                        <a
                                                            href="#!"
                                                            class="level-right"
                                                            on:click=move |_| {
                                                                delete_activity
                                                                    .dispatch(DeleteActivity {
                                                                        activity_id: activity.id,
                                                                    });
                                                            }
                                                        >

                                                            <i class="material-symbols-rounded">delete</i>
                                                        </a>
                                                    </div>
                                                </div>
                                            </div>
                                        </For>
                                    }
                                        .into_view()
                                }
                            })
                    }}
                    <ActivityDetails activity=show_activity/>

                </ErrorBoundary>
            </Transition>
        </div>
    }
}

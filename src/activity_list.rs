#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::error_template::ErrorTemplate;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Local};
use leptos::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityListEntry {
    pub id: i64,
    pub start_time: DateTime<Local>,
    pub duration: BigDecimal,
}

#[server(ActivityList, "/api")]
pub async fn get_activity_list() -> Result<Vec<ActivityListEntry>, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::ServerError("Not logged in".to_string()));
    }
    let user = auth.current_user.unwrap();
    let pool = pool()?;
    let activities = query_as!(ActivityListEntry, r#"SELECT activities.id, activities.start_time, activities.duration  FROM activities WHERE activities.user_id = $1::bigint"#, user.id).fetch_all(&pool).await?;
    Ok(activities)
}

#[component]
pub fn ActivityList() -> impl IntoView {
    let activities = create_resource(move || (), move |_| get_activity_list());
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
                                        <ul>
                                            <For
                                                each=move || activities.clone()
                                                key=|e| e.id
                                                let:activity
                                            >
                                                <li>
                                                    {activity.id} ": " {format!("{}", activity.start_time)} " "
                                                    {format!("{}", activity.duration)} s
                                                </li>
                                            </For>
                                        </ul>
                                    }
                                        .into_view()
                                }
                            })
                    }}

                </ErrorBoundary>
            </Transition>
        </div>
    }
}

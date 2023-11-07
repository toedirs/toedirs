#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::error_template::ErrorTemplate;
use chrono::{DateTime, Duration, Local};
use leptos::*;
use leptos_charts::{Color, Gradient, PieChart, PieChartOptions, Series};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartrateSummary {
    pub zone1: Option<i64>,
    pub zone2: Option<i64>,
    pub zone3: Option<i64>,
    pub zone4: Option<i64>,
}
#[cfg(feature = "ssr")]
pub async fn heartrate_zone_summary(
    user_id: i64,
    from: DateTime<Local>,
    to: DateTime<Local>,
    executor: sqlx::PgPool,
) -> Result<HeartrateSummary, sqlx::Error> {
    let result = sqlx::query_as!(HeartrateSummary, r#"SELECT
    COUNT(*) FILTER (WHERE m.zone = 4) AS zone4,
    COUNT(*) FILTER (WHERE m.zone = 3) AS zone3,
    COUNT(*) FILTER (WHERE m.zone = 2) AS zone2,
    COUNT(*) FILTER (WHERE m.zone = 1) AS zone1
FROM (
    SELECT record.heartrate,
        CASE
            WHEN record.heartrate >= 126 AND record.heartrate < 142 THEN 1
            WHEN record.heartrate >= 142 AND record.heartrate < 158 THEN 2
            WHEN record.heartrate >= 158 AND record.heartrate < 171 THEN 3
            WHEN record.heartrate >= 171 THEN 4
        END as zone
    FROM activities as activities
    LEFT JOIN records as record ON record.activity_id = activities.id
    WHERE activities.user_id = $1::bigint AND activities.start_time >= $2::timestamptz AND activities.end_time <= $3::timestamptz
        AND record.heartrate IS NOT NULL AND record.heartrate >= 126
) m
"#, &user_id, &from,&to).fetch_one(&executor).await?;
    Ok(result)
}

#[server(HeartrateSummaryAction, "/api")]
pub async fn heartrate_zone_summary_action(
    from: DateTime<Local>,
    to: DateTime<Local>,
) -> Result<HeartrateSummary, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::ServerError("Not logged in".to_string()));
    }
    let user = auth.current_user.unwrap();
    let pool = pool()?;
    let summary = heartrate_zone_summary(user.id, from, to, pool).await?;
    Ok(summary)
}

#[component]
pub fn HeartrateSummaryChart() -> impl IntoView {
    let zone_summary = create_resource(
        move || (),
        move |_| heartrate_zone_summary_action(Local::now() - Duration::days(120), Local::now()),
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                {move || {
                    zone_summary
                        .get()
                        .map(move |zone_summary| match zone_summary {
                            Err(e) => {
                                view! { <pre class="error">"Error: " {e.to_string()}</pre> }
                                    .into_view()
                            }
                            Ok(zone_summary) => {
                                let data:Series<i64> = vec![
                                    (zone_summary.zone1.unwrap_or(0),"Zone 1".to_string()),
                                    (zone_summary.zone2.unwrap_or(0),"Zone 2".to_string()),
                                    (zone_summary.zone3.unwrap_or(0),"Zone 3".to_string()),
                                    (zone_summary.zone4.unwrap_or(0),"Zone 4".to_string())
                                ].into();
                                let options: Box<PieChartOptions> = Box::new(PieChartOptions { color: Box::new(Gradient {from: Color::RGB(0,255,0), to:Color::RGB(255,0,0)}) });
                                view! {
                                    <PieChart
                                        values=data.into()
                                        options=options
                                        attr:width="100%"
                                        attr:height="100%"
                                    />
                                }
                                    .into_view()
                            }
                        })
                }}

            </ErrorBoundary>
        </Transition>
    }
}

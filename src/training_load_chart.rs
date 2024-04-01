#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
#[cfg(feature = "ssr")]
use chrono::Duration;
use chrono::{DateTime, Local};
use leptos::*;
use leptos_charts::{BarChart, BarChartOptions};
use serde::{Deserialize, Serialize};

use crate::error_template::ErrorTemplate;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingLoad {
    pub load: i64,
    pub date: DateTime<Local>,
}

#[cfg(feature = "ssr")]
pub async fn training_load(
    user_id: i64,
    from: DateTime<Local>,
    to: DateTime<Local>,
    executor: sqlx::PgPool,
) -> Result<Vec<TrainingLoad>, sqlx::Error> {
    //todo: calculate actual trimp
    let result:Vec<TrainingLoad> = sqlx::query_as!(TrainingLoad, r#" 
    WITH weeks as (
        SELECT generate_series(
            date_trunc('week', $2::timestamptz),
            date_trunc('week', $3::timestamptz),
            '1 week'
        ) as start
    )
    SELECT
        weeks.start as "date!",
        COALESCE(SUM(activities.load), 0)::int8 as "load!"
    FROM weeks
    LEFT JOIN (
        SELECT 
            COALESCE(ROUND((AVG(record.heartrate)-125)/8 * EXTRACT(EPOCH FROM (activities.end_time - activities.start_time))/60),0)::int8 as load,
            date_trunc('week', activities.start_time ) as date
        FROM activities as activities
        LEFT JOIN records as record ON record.activity_id = activities.id
        WHERE activities.user_id = $1::bigint 
            AND record.heartrate IS NOT NULL AND record.heartrate >= 126
        GROUP BY activities.id
    ) activities ON activities.date = weeks.start
    GROUP BY weeks.start
    ORDER BY weeks.start
"#, &user_id, &from,&to).fetch_all(&executor).await?;
    Ok(result)
}

#[server(TrainingLoadaction, "/api")]
pub async fn training_load_action(
    from: Option<DateTime<Local>>,
    to: Option<DateTime<Local>>,
) -> Result<Vec<TrainingLoad>, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::new("Not logged in".to_string()));
    }
    let user = auth.current_user.expect("the user to be logged in");
    let pool = pool()?;
    let summary = training_load(
        user.id,
        from.unwrap_or(Local::now() - Duration::try_days(120).unwrap()),
        to.unwrap_or(Local::now()),
        pool,
    )
    .await?;
    Ok(summary)
}

#[component]
pub fn TrainingLoadChart(
    #[prop(into)] from: Memo<Option<DateTime<Local>>>,
    #[prop(into)] to: Memo<Option<DateTime<Local>>>,
) -> impl IntoView {
    let training_load = create_resource(
        move || (from(), to()),
        move |_| training_load_action(from(), to()),
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                {move || {
                    training_load
                        .get()
                        .map(move |training_load| match training_load {
                            Err(e) => {
                                view! { <pre class="error">"Error: " {e.to_string()}</pre> }
                                    .into_view()
                            }
                            Ok(training_load) => {
                                let data: Vec<_> = training_load.iter().map(|t| t.load).collect();
                                let options: Box<BarChartOptions> = Box::new(BarChartOptions {
                                    max_ticks: 5,
                                    ..Default::default()
                                });
                                view! {
                                    // (t.load, t.date.format("%Y-%m-%d").to_string())).collect();

                                    <BarChart
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

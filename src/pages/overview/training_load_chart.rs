#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use charming::{
    component::{Axis, Grid},
    element::{AxisType, Tooltip, Trigger},
    series::Bar,
    Chart, WasmRenderer,
};
#[cfg(feature = "ssr")]
use chrono::Duration;
use chrono::{DateTime, Local};
use leptos::{html::Div, *};
use leptos_use::{use_element_size, UseElementSizeReturn};
use serde::{Deserialize, Serialize};
use std::cmp;

use crate::{app::FitFileUploaded, error_template::ErrorTemplate};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    let result: Vec<TrainingLoad> = sqlx::query_as!(
        TrainingLoad,
        r#" 
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
            activities.load as load,
            date_trunc('week', activities.start_time ) as date
        FROM activities as activities
        LEFT JOIN records as record ON record.activity_id = activities.id
        WHERE activities.user_id = $1::bigint 
            AND record.heartrate IS NOT NULL
        GROUP BY activities.id
    ) activities ON activities.date = weeks.start
    GROUP BY weeks.start
    ORDER BY weeks.start
"#,
        &user_id,
        &from,
        &to
    )
    .fetch_all(&executor)
    .await?;
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
    let uploaded = use_context::<FitFileUploaded>().unwrap();
    let training_load = create_resource(
        move || (from(), to(), uploaded.0()),
        move |(from, to, _)| training_load_action(from, to),
    );
    let trainingload_chart = create_node_ref::<Div>();
    let UseElementSizeReturn { width, height: _ } = use_element_size(trainingload_chart);
    let _chart = create_local_resource(
        move || (training_load.get(), width()),
        move |(load, width)| async move {
            if let Some(Ok(training_load)) = load {
                let chart = Chart::new()
                    .grid(Grid::new().top(10).bottom(20))
                    .tooltip(Tooltip::new().trigger(Trigger::Axis))
                    .x_axis(
                        Axis::new().type_(AxisType::Category).data(
                            training_load
                                .iter()
                                .map(|t| format!("{}", t.date.format("%Y-%m-%d")))
                                .collect::<Vec<_>>(),
                        ),
                    )
                    .y_axis(Axis::new().type_(AxisType::Value))
                    .series(
                        Bar::new().data(training_load.iter().map(|t| t.load).collect::<Vec<_>>()),
                    );
                let renderer = WasmRenderer::new(cmp::max(width as u32, 300), 155);
                let _rendered = renderer.render("training_load_chart", &chart);
            }
        },
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                <div node_ref=trainingload_chart id="training_load_chart"></div>

            </ErrorBoundary>
        </Transition>
    }
}

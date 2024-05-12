#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use charming::{
    component::{Axis, Grid},
    element::{AxisType, Tooltip, Trigger},
    series::Line,
    Chart, WasmRenderer,
};
use chrono::{DateTime, Days, Duration, Local};
use itertools::MultiUnzip;
use leptos::{html::Div, *};
use leptos_use::{use_element_size, UseElementSizeReturn};
use serde::{Deserialize, Serialize};
use std::{cmp, f64::consts};

use crate::{app::FitFileUploaded, error_template::ErrorTemplate};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingLoad {
    pub load: i64,
    pub date: DateTime<Local>,
}

#[cfg(feature = "ssr")]
pub async fn daily_training_load(
    user_id: i64,
    to: DateTime<Local>,
    executor: sqlx::PgPool,
) -> Result<Vec<TrainingLoad>, sqlx::Error> {
    let result: Vec<TrainingLoad> = sqlx::query_as!(
        TrainingLoad,
        r#"
            SELECT 
                SUM(a.load) as "load!",
                a.date as "date!"
            FROM (
                SELECT 
                    COALESCE(activities.load, 0) as load,
                    d.dt as date
                FROM
                (
                    SELECT 
                        dt
                    FROM
                        generate_series(
                            (SELECT date_trunc('day', MIN(a.start_time)) from activities a WHERE a.user_id=$1::bigint),
                            date_trunc('day', $2::timestamptz) + interval '1' day,
                            '1 day') dt
                ) d
                
                LEFT JOIN activities on date_trunc('day',activities.start_time) = d.dt AND activities.user_id=$1::bigint
             ) a
            
            GROUP BY a.date
            ORDER BY a.date ASC
            "#
,
        &user_id,
        &to
    )
    .fetch_all(&executor)
    .await?;
    Ok(result)
}

#[server]
pub async fn daily_training_load_action(
    to: Option<DateTime<Local>>,
) -> Result<Vec<TrainingLoad>, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::new("Not logged in".to_string()));
    }
    let user = auth.current_user.expect("the user to be logged in");
    let pool = pool()?;
    let summary = daily_training_load(user.id, to.unwrap_or(Local::now()), pool).await?;
    Ok(summary)
}

#[component]
pub fn FitnessLevelChart(
    #[prop(into)] from: Memo<Option<DateTime<Local>>>,
    #[prop(into)] to: Memo<Option<DateTime<Local>>>,
) -> impl IntoView {
    let uploaded = use_context::<FitFileUploaded>().unwrap();
    let training_load = create_resource(
        move || (to.get(), uploaded.0()),
        move |(to, _)| daily_training_load_action(to),
    );
    let fitness_chart = create_node_ref::<Div>();
    let UseElementSizeReturn { width, height: _ } = use_element_size(fitness_chart);
    let _chart = create_local_resource(
        move || (from.get(), to.get(), training_load.get(), width()),
        move |(from, to, training_load, width)| async move {
            if let Some(Ok(training_load)) = training_load {
                let from = from.unwrap_or(Local::now() - Duration::try_days(120).unwrap());
                let to = to
                    .unwrap_or(Local::now())
                    .checked_add_days(Days::new(2))
                    .unwrap();
                let moving_average_historic = consts::E.powf(-1.0 / 42.0); //moving average for last 42 days
                let moving_average_acute = consts::E.powf(-1.0 / 7.0); //moving average for last 7 days
                let (date, historic_load, acute_load, intensity): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
                    training_load
                        .iter()
                        .scan((0.0, 0.0), |acc, x| {
                            *acc = (
                                acc.0 * moving_average_historic
                                    + x.load as f64 * (1.0 - moving_average_historic),
                                acc.1 * moving_average_acute
                                    + x.load as f64 * (1.0 - moving_average_acute),
                            );
                            Some((x.date, acc.0, acc.1, acc.1 - acc.0))
                        })
                        .filter(|t| t.0 >= from && t.0 <= to)
                        .map(|e| {
                            (
                                format!("{}", e.0.format("%Y-%m-%d")),
                                e.1.round() as i32,
                                e.2.round() as i32,
                                e.3.round() as i32,
                            )
                        })
                        .multiunzip();

                let chart = Chart::new()
                    .grid(Grid::new().top(10).bottom(20))
                    .tooltip(Tooltip::new().trigger(Trigger::Axis))
                    .x_axis(Axis::new().type_(AxisType::Category).data(date))
                    .y_axis(Axis::new().type_(AxisType::Value))
                    .series(
                        Line::new()
                            .name("Fitness")
                            .show_symbol(false)
                            .data(historic_load),
                    )
                    .series(
                        Line::new()
                            .name("Acute Load")
                            .show_symbol(false)
                            .data(acute_load),
                    )
                    .series(
                        Line::new()
                            .name("Intensity")
                            .show_symbol(false)
                            .data(intensity),
                    );
                let renderer = WasmRenderer::new(cmp::max(width as u32, 300), 155);
                let _rendered = renderer.render("fitness_chart", &chart);
            }
        },
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                <div node_ref=fitness_chart id="fitness_chart"></div>
            </ErrorBoundary>
        </Transition>
    }
}

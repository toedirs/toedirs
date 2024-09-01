#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::{
    app::FitFileUploaded, error_template::ErrorTemplate, models::slope_speed::HeartrateZone,
};
use charming::{
    component::{Axis, Grid, Legend},
    element::{Tooltip, Trigger},
    series::Scatter,
    Chart, WasmRenderer,
};
#[cfg(feature = "ssr")]
use chrono::Duration;
use chrono::{DateTime, Local};
use leptos::{html::Div, *};
use leptos_use::{use_element_size, UseElementSizeReturn};
use serde::{Deserialize, Serialize};
use std::cmp;

#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlopeSpeed {
    pub slope: f64,
    pub speed: f64,
    pub zone: HeartrateZone,
}
#[cfg(feature = "ssr")]
impl sqlx::FromRow<'_, PgRow> for SlopeSpeed {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            slope: row.get("slope"),
            speed: row.get("speed"),
            zone: HeartrateZone::from_str(&row.get::<&str, _>("zone")).unwrap(),
        })
    }
}

#[cfg(feature = "ssr")]
pub async fn slope_speed(
    user_id: i64,
    from: DateTime<Local>,
    to: DateTime<Local>,
    executor: sqlx::PgPool,
) -> Result<Vec<SlopeSpeed>, sqlx::Error> {
    let result: Vec<SlopeSpeed> = sqlx::query_as(
        r#"
            SELECT
                sp.slope::float as slope,
                AVG(sp.average_speed)::float as speed,
                sp.heartrate_zone::text as zone
            FROM slope_speed sp
            WHERE sp.user_id = $1::bigint and sp.start_time >= $2::timestamptz and sp.start_time <= $3::timestamptz
            GROUP BY sp.slope, sp.heartrate_zone
        "#)
         .bind(&user_id)
         .bind(&from )
         .bind(&to)
    .fetch_all(&executor).await?;
    Ok(result)
}

#[server(SlopeSpeedAction, "/api")]
pub async fn slope_speed_action(
    from: Option<DateTime<Local>>,
    to: Option<DateTime<Local>>,
) -> Result<Vec<SlopeSpeed>, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::new("Not logged in".to_string()));
    }
    let user = auth.current_user.expect("the user to be logged in");
    let pool = pool()?;
    let summary = slope_speed(
        user.id,
        from.unwrap_or(Local::now() - Duration::try_days(120).unwrap()),
        to.unwrap_or(Local::now()),
        pool,
    )
    .await?;
    Ok(summary)
}
#[component]
pub fn SlopeSpeedChart(
    #[prop(into)] from: Memo<Option<DateTime<Local>>>,
    #[prop(into)] to: Memo<Option<DateTime<Local>>>,
) -> impl IntoView {
    let uploaded = use_context::<FitFileUploaded>().unwrap();
    let slope_speed = create_resource(
        move || (from(), to(), uploaded.0()),
        move |(from, to, _)| slope_speed_action(from, to),
    );
    let slope_speed_chart = create_node_ref::<Div>();
    let UseElementSizeReturn { width, height: _ } = use_element_size(slope_speed_chart);
    let _chart = create_local_resource(
        move || (slope_speed.get(), width()),
        move |(slope_speed, width)| async move {
            if let Some(Ok(slope_speed)) = slope_speed {
                let chart = Chart::new()
                    .grid(Grid::new().top(20).bottom(20))
                    .x_axis(Axis::new())
                    .y_axis(Axis::new().scale(true))
                    .legend(Legend::new())
                    .tooltip(Tooltip::new().trigger(Trigger::Item))
                    .series(
                        Scatter::new().name("Zone 1").symbol_size(10).data(
                            slope_speed
                                .iter()
                                .filter(|s| s.zone == HeartrateZone::Zone1)
                                .map(|s| vec![s.slope, s.speed])
                                .collect(),
                        ),
                    )
                    .series(
                        Scatter::new().name("Zone 2").symbol_size(10).data(
                            slope_speed
                                .iter()
                                .filter(|s| s.zone == HeartrateZone::Zone2)
                                .map(|s| vec![s.slope, s.speed])
                                .collect(),
                        ),
                    )
                    .series(
                        Scatter::new().name("Zone 3").symbol_size(10).data(
                            slope_speed
                                .iter()
                                .filter(|s| s.zone == HeartrateZone::Zone3)
                                .map(|s| vec![s.slope, s.speed])
                                .collect(),
                        ),
                    );
                let renderer = WasmRenderer::new(cmp::max(width as u32, 300), 155);
                let _rendered = renderer.render("slope_speed_chart", &chart);
            }
        },
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                <div node_ref=slope_speed_chart id="slope_speed_chart"></div>

            </ErrorBoundary>
        </Transition>
    }
}

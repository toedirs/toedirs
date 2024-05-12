#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::{app::FitFileUploaded, error_template::ErrorTemplate};
use charming::{
    component::{Grid, Legend},
    datatype::DataPointItem,
    df,
    element::{ItemStyle, Orient, Tooltip, Trigger},
    series::Pie,
    Chart, WasmRenderer,
};
#[cfg(feature = "ssr")]
use chrono::Duration;
use chrono::{DateTime, Local};
use leptos::{html::Div, *};
use leptos_use::{use_element_size, UseElementSizeReturn};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::*;
use std::cmp;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeartrateSummary {
    pub zone1: Option<i64>,
    pub zone2: Option<i64>,
    pub zone3: Option<i64>,
}
#[cfg(feature = "ssr")]
pub async fn heartrate_zone_summary(
    user_id: i64,
    from: DateTime<Local>,
    to: DateTime<Local>,
    executor: sqlx::PgPool,
) -> Result<HeartrateSummary, sqlx::Error> {
    let result = sqlx::query_as!(HeartrateSummary, r#"SELECT
    COUNT(*) FILTER (WHERE m.zone = 3) AS zone3,
    COUNT(*) FILTER (WHERE m.zone = 2) AS zone2,
    COUNT(*) FILTER (WHERE m.zone = 1) AS zone1
FROM (
    SELECT record.heartrate,
        CASE
            WHEN record.heartrate >= COALESCE(up.max_heartrate * 0.55, 100) AND record.heartrate < COALESCE(up.aerobic_threshold,155) THEN 1
            WHEN record.heartrate >= COALESCE(up.aerobic_threshold, 155) AND record.heartrate < COALESCE(up.anaerobic_threshold,172) THEN 2
            WHEN record.heartrate >= COALESCE(up.anaerobic_threshold, 172) THEN 3
        END as zone
    FROM activities as activities
    LEFT JOIN records as record ON record.activity_id = activities.id
    LEFT JOIN user_preferences up ON up.user_id=activities.user_id
    WHERE activities.user_id = $1::bigint AND activities.start_time >= $2::timestamptz AND activities.end_time <= $3::timestamptz
        AND record.heartrate IS NOT NULL AND record.heartrate >= COALESCE(up.max_heartrate * 0.55, 100)
) m
"#, &user_id, &from,&to).fetch_one(&executor).await?;
    Ok(result)
}

#[server(HeartrateSummaryAction, "/api")]
pub async fn heartrate_zone_summary_action(
    from: Option<DateTime<Local>>,
    to: Option<DateTime<Local>>,
) -> Result<HeartrateSummary, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::new("Not logged in".to_string()));
    }
    let user = auth.current_user.expect("the user to be logged in");
    let pool = pool()?;
    let summary = heartrate_zone_summary(
        user.id,
        from.unwrap_or(Local::now() - Duration::try_days(120).unwrap()),
        to.unwrap_or(Local::now()),
        pool,
    )
    .await?;
    Ok(summary)
}

#[component]
pub fn HeartrateZoneSummaryChart(
    #[prop(into)] from: Memo<Option<DateTime<Local>>>,
    #[prop(into)] to: Memo<Option<DateTime<Local>>>,
) -> impl IntoView {
    let uploaded = use_context::<FitFileUploaded>().unwrap();
    let zone_summary = create_resource(
        move || (from(), to(), uploaded.0()),
        move |(from, to, _)| heartrate_zone_summary_action(from, to),
    );
    let heartrate_summary_chart = create_node_ref::<Div>();
    let UseElementSizeReturn { width, height: _ } = use_element_size(heartrate_summary_chart);
    let _chart = create_local_resource(
        move || (zone_summary.get(), width()),
        move |(zone_summary, width)| async move {
            if let Some(Ok(zone_summary)) = zone_summary {
                let chart = Chart::new()
                    .grid(Grid::new().top(10).bottom(10))
                    .legend(Legend::new().orient(Orient::Vertical).left("left"))
                    .tooltip(Tooltip::new().trigger(Trigger::Item))
                    .series(Pie::new().radius("75%").data(df!(
                        DataPointItem::new(zone_summary.zone1.unwrap_or(0))
                            .name("Zone 1")
                            .item_style(ItemStyle::new().color("#a6da95")),
                        DataPointItem::new(zone_summary.zone2.unwrap_or(0))
                            .name( "Zone 2")
                            .item_style(ItemStyle::new().color("#eed49f")),
                        DataPointItem::new(zone_summary.zone3.unwrap_or(0))
                            .name( "Zone 3")
                            .item_style(ItemStyle::new().color("#ed8796")),
                    )));
                let renderer = WasmRenderer::new(cmp::max(width as u32, 300), 155);
                let _rendered = renderer.render("heartrate_summary_chart", &chart);
            }
        },
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                <div node_ref=heartrate_summary_chart id="heartrate_summary_chart"></div>

            </ErrorBoundary>
        </Transition>
    }
}

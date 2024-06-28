#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::{app::FitFileUploaded, error_template::ErrorTemplate, pages::user::get_preferences};
use charming::{
    component::{Axis, Grid, VisualMap, VisualMapPiece, VisualMapType},
    datatype::DataPointItem,
    element::{AxisType, ItemStyle, Tooltip, Trigger},
    series::Bar,
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
pub struct HeartrateDistributionEntry {
    pub heartrate: i32,
    pub count: i32,
}

#[cfg(feature = "ssr")]
pub async fn heartrate_zone_summary(
    user_id: i64,
    from: DateTime<Local>,
    to: DateTime<Local>,
    executor: sqlx::PgPool,
) -> Result<Vec<HeartrateDistributionEntry>, sqlx::Error> {
    let result = sqlx::query_as!(HeartrateDistributionEntry, r#"
    SELECT
        r.heartrate::int4 as "heartrate!",
        COALESCE(COUNT(*),0)::int4 as "count!"
    FROM activities a 
    JOIN records r on r.activity_id = a.id
    LEFT JOIN user_preferences up ON up.user_id=a.user_id
    WHERE a.user_id = $1::bigint AND a.start_time >= $2::timestamptz AND a.end_time <= $3::timestamptz
        AND r.heartrate IS NOT NULL AND r.heartrate >= COALESCE(up.max_heartrate * 0.55, 100)
    GROUP BY r.heartrate
    ORDER BY r.heartrate ASC
"#, &user_id, &from,&to).fetch_all(&executor).await?;
    Ok(result)
}

#[server()]
pub async fn heartrate_distribution_action(
    from: Option<DateTime<Local>>,
    to: Option<DateTime<Local>>,
) -> Result<Vec<HeartrateDistributionEntry>, ServerFnError> {
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
pub fn HeartrateDistributionChart(
    #[prop(into)] from: Memo<Option<DateTime<Local>>>,
    #[prop(into)] to: Memo<Option<DateTime<Local>>>,
) -> impl IntoView {
    let uploaded = use_context::<FitFileUploaded>().unwrap();
    let zone_distribution = create_resource(
        move || (from(), to(), uploaded.0()),
        move |(from, to, _)| heartrate_distribution_action(from, to),
    );
    let user_prefs = create_resource(move || (), |_| async move { get_preferences().await });
    let heartrate_distibution_chart = create_node_ref::<Div>();
    let UseElementSizeReturn { width, height: _ } = use_element_size(heartrate_distibution_chart);
    let _chart = create_local_resource(
        move || (zone_distribution.get(), user_prefs.get(), width()),
        move |(zone_distribution, user_prefs, width)| async move {
            let (aerobic, anaerobic, max_hr) = if let Some(Ok(prefs)) = user_prefs {
                (
                    prefs.aerobic_threshold,
                    prefs.anaerobic_threshold,
                    prefs.max_heartrate,
                )
            } else {
                (155, 173, 183)
            };
            if let Some(Ok(zone_distribution)) = zone_distribution {
                let chart = Chart::new()
                    .grid(Grid::new().top(10).bottom(20))
                    .tooltip(Tooltip::new().trigger(Trigger::Item))
                    .visual_map(
                        VisualMap::new()
                            .dimension(0)
                            .show(false)
                            .type_(VisualMapType::Piecewise)
                            .min(0)
                            .max(max_hr)
                            .pieces(vec![
                                VisualMapPiece::new()
                                    .gt(0)
                                    .lte(aerobic as f32 * 0.7)
                                    .color("#7dc4e4"),
                                VisualMapPiece::new()
                                    .gt(aerobic as f32 * 0.7)
                                    .lte(aerobic)
                                    .color("#a6da95"),
                                VisualMapPiece::new()
                                    .gt(aerobic)
                                    .lte(anaerobic)
                                    .color("#eed49f"),
                                VisualMapPiece::new()
                                    .gt(anaerobic)
                                    .lte(max_hr)
                                    .color("#ed8796"),
                            ]),
                    )
                    .x_axis(
                        Axis::new().type_(AxisType::Category).data(
                            zone_distribution
                                .iter()
                                .map(|d| format!("{}", d.heartrate))
                                .collect::<Vec<_>>(),
                        ),
                    )
                    .y_axis(Axis::new().type_(AxisType::Value))
                    .series(
                        Bar::new().data(
                            zone_distribution
                                .iter()
                                .map(|d| {
                                    DataPointItem::new(d.count as i64).item_style(
                                        ItemStyle::new().color(
                                            if d.heartrate < (aerobic as f32 * 0.7) as i32 {
                                                "#7dc4e4"
                                            } else if d.heartrate < aerobic {
                                                "#a6da95"
                                            } else if d.heartrate < anaerobic {
                                                "#eed49f"
                                            } else {
                                                "#ed8796"
                                            },
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        ),
                    );
                let renderer = WasmRenderer::new(cmp::max(width as u32, 300), 155);
                let _rendered = renderer.render("heartrate_distribution_chart", &chart);
            }
        },
    );

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors/> }
            }>
                <div node_ref=heartrate_distibution_chart id="heartrate_distribution_chart"></div>

            </ErrorBoundary>
        </Transition>
    }
}

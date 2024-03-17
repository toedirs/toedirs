use std::time::Duration;

#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Local};
use humantime::format_duration;
use leptos::*;
use leptos_charts::{Color, Gradient, LineChart, LineChartOptions};
use leptos_leaflet::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub struct Lap {
    pub id: i64,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub distance: Option<f64>,
    pub calories: Option<i16>,
    pub average_heartrate: Option<i16>,
    pub min_heartrate: Option<i16>,
    pub max_heartrate: Option<i16>,
    pub sport: Option<String>,
    pub ascent: Option<i16>,
    pub descent: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub struct Record {
    pub timestamp: DateTime<Local>,
    pub heartrate: Option<i16>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub distance: Option<f64>,
    pub speed: Option<f64>,
    pub altitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetail {
    pub id: i64,
    pub start_time: DateTime<Local>,
    pub duration: BigDecimal,
    pub sport: String,
    pub laps: Option<Vec<Lap>>,
    pub records: Vec<Record>,
}

#[server]
pub async fn activity_details(id: i64) -> Result<ActivityDetail, ServerFnError> {
    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ServerFnError::new("Not logged in".to_string()));
    }
    let user = auth.current_user.unwrap();
    let pool = pool()?;
    let activity_detail = query_as!(
        ActivityDetail,
        r#"
        SELECT 
            activities.id, 
            activities.start_time, 
            activities.duration,
            COALESCE(string_agg(sessions.sport,', '),'General') as "sport!",
            (
                SELECT
                    ARRAY_AGG(
                        (
                            laps.id, 
                            laps.start_time, 
                            laps.end_time, 
                            laps.distance::float8, 
                            laps.calories, 
                            laps.average_heartrate, 
                            laps.min_heartrate, 
                            laps.max_heartrate, 
                            laps.sport,
                            laps.ascent, 
                            laps.descent
                        )
                    )
                FROM laps
                WHERE laps.activity_id = $2::bigint
            ) as "laps:Vec<Lap>",
            (
                SELECT
                    ARRAY_AGG(
                        (
                            records.timestamp,
                            records.heartrate,
                            records.latitude,
                            records.longitude,
                            records.distance::float8,
                            records.speed::float8,
                            records.altitude::float8
                        )
                        ORDER BY records.timestamp ASC
                    ) 
                FROM records
                WHERE records.activity_id = $2::bigint
            ) as "records!:Vec<Record>"
        FROM activities 
        JOIN sessions on sessions.activity_id=activities.id
        WHERE activities.user_id = $1::bigint AND activities.id = $2::bigint
        GROUP BY activities.id
        "#,
        user.id,
        id
    )
    .fetch_one(&pool)
    .await?;
    Ok(activity_detail)
}

#[component]
pub fn ActivityDetails(activity: RwSignal<Option<i64>>) -> impl IntoView {
    let close = move |_| activity.set(None);
    let detail = create_resource(activity, |id| async move {
        if let Some(id) = id {
            activity_details(id).await.ok()
        } else {
            None
        }
    });
    view! {
        <Show when=move || { activity().is_some() } fallback=|| {}>

            <div class="modal is-active">
                <div class="modal-background" on:click=close></div>
                <Suspense fallback=move || {
                    view! { <div class="modal-content">Loading...</div> }
                }>
                    {move || {
                        detail
                            .get()
                            .map(|detail| match detail {
                                None => view! { <pre>"Error"</pre> }.into_view(),
                                Some(detail) => {
                                    let data = detail
                                        .records
                                        .iter()
                                        .filter_map(|r| {
                                            r.heartrate.map(|h| (r.timestamp.timestamp(), h))
                                        })
                                        .collect::<Vec<(i64, i16)>>();
                                    view! {
                                        <div class="modal-card is-full">
                                            <div class="modal-card-head">
                                                <div class="modal-card-title">
                                                    <p class="title is-4">{detail.sport}</p>
                                                    <p class="subtitle is-6">
                                                        {detail.start_time.format("%Y-%m-%d").to_string()} ,
                                                        {format_duration(
                                                                Duration::new(detail.duration.to_u64().unwrap(), 0),
                                                            )
                                                            .to_string()}
                                                    </p>
                                                </div>
                                                <button
                                                    class="delete"
                                                    aria-label="close"
                                                    on:click=close
                                                ></button>
                                            </div>
                                            <div class="modal-card-body">
                                                <div class="columns">
                                                    <div class="column">
                                                        <LineChart
                                                            values=data.into()
                                                            options=Box::new(LineChartOptions {
                                                                color: Box::new(Gradient {
                                                                    from: Color::RGB(0, 255, 0),
                                                                    to: Color::RGB(255, 0, 0),
                                                                }),
                                                                ..Default::default()
                                                            })

                                                            attr:style="width:100%;height:100%;min-height:500px;"
                                                        />

                                                    </div>

                                                    {
                                                        let coordinates: Option<Vec<(f64, f64)>> = detail
                                                            .records
                                                            .into_iter()
                                                            .filter_map(|r| {
                                                                r.latitude.map(|lat| r.longitude.map(|long| (lat, long)))
                                                            })
                                                            .collect();
                                                        match coordinates {
                                                            Some(coordinates) if coordinates.len() > 0 => {
                                                                let num_coords = coordinates.len();
                                                                let center = coordinates
                                                                    .clone()
                                                                    .into_iter()
                                                                    .fold(
                                                                        (0.0, 0.0),
                                                                        |acc, pos| (acc.0 + pos.0, acc.1 + pos.1),
                                                                    );
                                                                let center = (
                                                                    center.0 / num_coords as f64,
                                                                    center.1 / num_coords as f64,
                                                                );
                                                                view! {
                                                                    <div class="column is-half">
                                                                        <MapContainer
                                                                            style="height:500px;"
                                                                            center=Position::new(center.0, center.1)
                                                                            zoom=13.0
                                                                            set_view=true
                                                                        >
                                                                            <TileLayer url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"/>
                                                                            <Polyline positions=positions(&coordinates)/>

                                                                        </MapContainer>
                                                                    </div>
                                                                }
                                                                    .into_view()
                                                            }
                                                            _ => view! {}.into_view(),
                                                        }
                                                    }

                                                </div>
                                                <div class="columns">
                                                    <div class="column is-fullwidth">
                                                        <table class="table is-striped is-hoverable is-fullwidth">
                                                            <thead>
                                                                <tr>
                                                                    <th>Lap</th>
                                                                    <th>Time</th>
                                                                    <th>Distance</th>
                                                                    <th>Avg. Heartrate</th>
                                                                    <th>Calories</th>
                                                                    <th>Ascent</th>
                                                                    <th>Descent</th>

                                                                </tr>
                                                            </thead>
                                                            <tbody>

                                                                {match detail.laps {
                                                                    Some(laps) => {
                                                                        view! {
                                                                            <For each=move || laps.clone() key=|l| l.id let:lap>
                                                                                <tr>
                                                                                    <td></td>
                                                                                    <td>
                                                                                        {format_duration(
                                                                                                (lap.end_time - lap.start_time)
                                                                                                    .to_std()
                                                                                                    .expect("couldn't convert duration"),
                                                                                            )
                                                                                            .to_string()}
                                                                                    </td>
                                                                                    <td>{lap.distance}</td>
                                                                                    <td>{lap.average_heartrate}</td>
                                                                                    <td>{lap.calories}</td>
                                                                                    <td>{lap.ascent}</td>
                                                                                    <td>{lap.descent}</td>
                                                                                </tr>
                                                                            </For>
                                                                        }
                                                                            .into_view()
                                                                    }
                                                                    None => {
                                                                        view! {
                                                                            <tr>
                                                                                <td>1</td>
                                                                                <td>
                                                                                    {format_duration(
                                                                                            Duration::new(detail.duration.to_u64().unwrap(), 0),
                                                                                        )
                                                                                        .to_string()}
                                                                                </td>
                                                                                <td></td>
                                                                                <td></td>
                                                                                <td></td>
                                                                                <td></td>
                                                                                <td></td>
                                                                            </tr>
                                                                        }
                                                                            .into_view()
                                                                    }
                                                                }}

                                                            </tbody>
                                                        </table>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                        .into_view()
                                }
                            })
                    }}

                </Suspense>
            </div>
        </Show>
    }
}

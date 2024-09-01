#[cfg(feature = "ssr")]
use super::{
    base::{DatabaseEntry, ModelError, New},
    record::Record,
    session::Session,
    user_preferences::UserPreferences,
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Serialize, Deserialize, Display, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "heartrate_zone", rename_all = "snake_case")
)]
#[derive(strum::EnumString)]
pub enum HeartrateZone {
    #[strum(serialize = "zone1")]
    Zone1,
    #[strum(serialize = "zone2")]
    Zone2,
    #[strum(serialize = "zone3")]
    Zone3,
}

impl TryFrom<String> for HeartrateZone {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "zone1" => Ok(Self::Zone1),
            "zone2" => Ok(Self::Zone2),
            "zone3" => Ok(Self::Zone3),
            _ => Err("Couldn't parse heartrate zone".to_string()),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SlopeSpeed {
    pub user_id: i64,

    pub start_time: DateTime<Local>,
    pub sport: Option<String>,
    pub slope: f64,
    pub average_speed: f64,
    pub heartrate_zone: HeartrateZone,
}
#[cfg(feature = "ssr")]
pub fn slope_speed_from_records(
    values: Vec<DatabaseEntry<New, Record>>,
    sessions: &Vec<DatabaseEntry<New, Session>>,
    user_id: i64,
    user_preferences: &UserPreferences,
) -> Result<SlopeSpeed, ModelError> {
    let filtered_values: Vec<&DatabaseEntry<New, Record>> = values
        .iter()
        .filter(|r| {
            r.state.distance.is_some()
                && r.state.altitude.is_some()
                && r.state.speed.is_some()
                && r.state.heartrate.is_some()
        })
        .collect();
    if filtered_values.len() < 2 {
        return Err(ModelError::ParseError("To few entries".to_string()));
    }
    let first = filtered_values.first().unwrap();
    let last = filtered_values.last().unwrap();
    if first.state.distance == last.state.distance {
        return Err(ModelError::ParseError(
            "No distance covered in range".to_string(),
        ));
    }
    let slope = (last.state.altitude.unwrap() - first.state.altitude.unwrap())
        / (last.state.distance.unwrap() - first.state.distance.unwrap());
    let slope = (slope * 20.0).round() / 20.0;
    if slope > 1.0 || slope < -1.0 {
        return Err(ModelError::ParseError(
            "Slope outside of valid range".to_string(),
        ));
    }
    let avg_speed = filtered_values
        .iter()
        .map(|r| r.state.speed.unwrap())
        .sum::<f64>()
        / filtered_values.len() as f64;
    let avg_speed = ((avg_speed * 1000.0).round() / 1000.0).clamp(-99.999, 99.999);
    let avg_heartrate = filtered_values
        .iter()
        .map(|r| r.state.heartrate.unwrap() as i32)
        .sum::<i32>()
        / filtered_values.len() as i32;
    let hr_zone = if avg_heartrate < user_preferences.aerobic_threshold.into() {
        HeartrateZone::Zone1
    } else if avg_heartrate >= user_preferences.anaerobic_threshold.into() {
        HeartrateZone::Zone3
    } else {
        HeartrateZone::Zone2
    };
    let sport = sessions
        .iter()
        .find(|s| {
            s.state.start_time <= first.state.timestamp && s.state.end_time > first.state.timestamp
        })
        .map_or(None, |s| s.state.sport.clone());
    Ok(SlopeSpeed {
        user_id,
        start_time: first.state.timestamp,
        sport,
        slope,
        average_speed: avg_speed,
        heartrate_zone: hr_zone,
    })
}

#[cfg(feature = "ssr")]
pub async fn insert_slopes(
    slopes: Vec<DatabaseEntry<New, SlopeSpeed>>,
    activity_id: i64,
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<(), ModelError> {
    use itertools::Itertools;

    let num_slopes = slopes.len();
    let activity_ids: Vec<i64> = std::iter::repeat(activity_id).take(num_slopes).collect();
    let (user_id, start_time, sport, slope, average_speed, heartrate_zone): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = slopes
        .clone()
        .into_iter()
        .map(|s| {
            (
                s.state.user_id,
                s.state.start_time,
                s.state.sport,
                s.state.slope,
                s.state.average_speed,
                format!("{}", s.state.heartrate_zone),
            )
        })
        .multiunzip();
    sqlx::query!(r#"
            INSERT INTO slope_speed(activity_id,user_id,start_time,sport,slope,average_speed,heartrate_zone)
            SELECT *
            FROM UNNEST($1::bigint[],$2::bigint[],$3::timestamptz[],$4::varchar[],$5::float8[],$6::float8[],$7::heartrate_zone[])
        "#,
        &activity_ids[..],
        &user_id[..],
        &start_time[..],
        &sport[..] as _,
        &slope[..],
        &average_speed[..],
        &heartrate_zone[..] as _
    ).execute(executor).await.map_err(|e|ModelError::InsertError(format!("Couldn't insert slope speed:{}:{:?}",e,slopes)))?;
    Ok(())
}

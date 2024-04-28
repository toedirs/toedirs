

use chrono::{DateTime, Local};
use fitparser::{profile::MesgNum, FitDataRecord, Value};
#[cfg(feature = "ssr")]
use itertools::Itertools;

use super::base::{DatabaseEntry, ModelError, New};



#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Session {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub sport: Option<String>,
    pub distance: Option<f64>,
    pub calories: Option<i32>,
    pub average_heartrate: Option<i16>,
    pub min_heartrate: Option<i16>,
    pub max_heartrate: Option<i16>,
    pub average_power: Option<i32>,
    pub ascent: Option<i32>,
    pub descent: Option<i32>,
    pub average_speed: Option<f64>,
    pub max_speed: Option<f64>,
}

impl TryFrom<FitDataRecord> for DatabaseEntry<New, Session> {
    type Error = ModelError;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Session => {}
            _ => return Err(ModelError::ParseError("Not a Session".to_string())),
        };
        let fields = value.fields();
        let start_time =
            fields
                .iter()
                .find(|&f| f.name() == "start_time")
                .ok_or(ModelError::ParseError(
                    "no start_time in record".to_string(),
                ))?;
        let start_time = match start_time.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => {
                return Err(ModelError::ParseError(
                    "start_time field is not a date".to_string(),
                ))
            }
        };

        let end_time = fields
            .iter()
            .find(|&f| f.name() == "timestamp")
            .ok_or(ModelError::ParseError("no end_time in record".to_string()))?;
        let end_time = match end_time.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => {
                return Err(ModelError::ParseError(
                    "end_time field is not a date".to_string(),
                ))
            }
        };

        let sport = fields.iter().find(|&f| f.name() == "sport");
        let sport = sport
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::String(val) => Some(val),
                _ => None,
            });

        let average_heartrate = fields.iter().find(|&f| f.name() == "avg_heart_rate");
        let average_heartrate = average_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(i16::from(val)),
                _ => None,
            });

        let min_heartrate = fields.iter().find(|&f| f.name() == "min_heart_rate");
        let min_heartrate = min_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(i16::from(val)),
                _ => None,
            });

        let max_heartrate = fields.iter().find(|&f| f.name() == "max_heart_rate");
        let max_heartrate = max_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(i16::from(val)),
                _ => None,
            });

        let calories = fields
            .iter()
            .find(|&f| f.name() == "calories" || f.name() == "total_calories");
        let calories = calories
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(i32::from(val)),
                _ => None,
            });

        let distance = fields
            .iter()
            .find(|&f| f.name() == "distance" || f.name() == "total_distance");
        let distance = distance
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::Float64(val) => Some(val),
                _ => None,
            });

        let ascent = fields
            .iter()
            .find(|&f| f.name() == "ascent" || f.name() == "total_ascent");
        let ascent = ascent
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(i32::from(val)),
                _ => None,
            });

        let descent = fields
            .iter()
            .find(|&f| f.name() == "descent" || f.name() == "total_descent");
        let descent = descent
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(i32::from(val)),
                _ => None,
            });

        let average_power = fields.iter().find(|&f| f.name() == "avg_power");
        let average_power = average_power
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(i32::from(val)),
                _ => None,
            });

        let average_speed = fields
            .iter()
            .find(|&f| f.name() == "avg_speed" || f.name() == "enhanced_avg_speed");
        let average_speed = average_speed
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::Float64(val) => Some(val),
                _ => None,
            });

        let max_speed = fields
            .iter()
            .find(|&f| f.name() == "max_speed" || f.name() == "enhanced_max_speed");
        let max_speed = max_speed
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::Float64(val) => Some(val),
                _ => None,
            });

        Ok(DatabaseEntry {
            state: Box::new(Session {
                start_time,
                end_time,
                sport,
                distance,
                calories,
                average_heartrate,
                min_heartrate,
                max_heartrate,
                ascent,
                descent,
                average_power,
                average_speed,
                max_speed,
            }),
            extra: New,
        })
    }
}
#[cfg(feature = "ssr")]
pub async fn insert_sessions(
    sessions: Vec<DatabaseEntry<New, Session>>,
    activity_id: i64,
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<(), ModelError> {
    let num_sessions = sessions.len();
    let activity_ids: Vec<i64> = std::iter::repeat(activity_id).take(num_sessions).collect();
    let (
    start_time,
    end_time,
    sport,
    distance,
    calories,
    average_heartrate,
    min_heartrate,
    max_heartrate,
    average_power,
    ascent,
    descent,
    average_speed,
    
    ): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = sessions.clone()
        .into_iter()
        .map(|r| {
            (
                r.state.start_time,
                r.state.end_time,
                r.state.sport,
                r.state.distance,
                r.state.calories,
                r.state.average_heartrate,
                r.state.min_heartrate,
                r.state.max_heartrate,
                r.state.average_power,
                r.state.ascent,
                r.state.descent,
                r.state.average_speed,
            )
        })
        .multiunzip();
    // itertools only supports up to 12 iterators, so we do this one separately
    let max_speed: Vec<_> = sessions.into_iter().map(|r|r.state.max_speed).collect();
    sqlx::query!(
        r#"
        INSERT INTO sessions(activity_id,start_time,end_time,sport,distance,calories,average_heartrate,min_heartrate,max_heartrate,average_power,ascent,descent,average_speed,max_speed)
        SELECT *
        FROM UNNEST($1::bigint[], $2::timestamptz[],$3::timestamptz[], $4::varchar[], $5::float8[], $6::int[], $7::smallint[], $8::smallint[], $9::smallint[], $10::int[], $11::int[], $12::int[], $13::float8[], $14::float8[])
        "#,
        &activity_ids[..],
        &start_time[..] as _,
        &end_time[..] as _,
        &sport[..] as _,
        &distance[..] as _,
        &calories[..] as _,
        &average_heartrate[..] as _,
        &min_heartrate[..] as _,
        &max_heartrate[..] as _,
        &average_power[..] as _,
        &ascent[..] as _,
        &descent[..] as _,
        &average_speed[..] as _,
        &max_speed[..] as _,
        
    ).execute(executor).await
        .map_err(|e| ModelError::InsertError(format!("Couldn't insert session: {}", e)))?;

    Ok(())
}

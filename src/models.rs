use chrono::{DateTime, Local, Duration};
use fitparser::{profile::MesgNum, FitDataRecord, Value};
#[cfg(feature = "ssr")]
use itertools::Itertools;
#[cfg(feature = "ssr")]
use sqlx::{query, Row};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("couldn't parse fit entry: {0}")]
    ParseError(String),
    #[error("couldn't insert entry into database: {0}")]
    InsertError(String),
}

#[derive(Debug, Clone)]
pub struct DatabaseEntry<S: DatabaseState, T> {
    pub state: Box<T>,
    pub extra: S,
}

#[derive(Debug, Clone)]
pub struct New;
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Stored {
    pub activity_id: i64,
}

pub trait DatabaseState {}
impl DatabaseState for New {}
impl DatabaseState for Stored {}

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub latitude: i32,
    pub longitude: i32,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Record {
    pub timestamp: DateTime<Local>,
    pub heartrate: Option<i16>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub distance: Option<f64>,
    pub speed: Option<f64>,
    pub altitude: Option<f64>,
}

impl TryFrom<FitDataRecord> for DatabaseEntry<New, Record> {
    type Error = ModelError;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Record => {}
            _ => return Err(ModelError::ParseError("Not a Record".to_string())),
        };
        let fields = value.fields();
        let timestamp = fields
            .iter()
            .find(|&f| f.name() == "timestamp")
            .ok_or(ModelError::ParseError("no timestamp in record".to_string()))?;
        let timestamp = match timestamp.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => {
                return Err(ModelError::ParseError(
                    "timestamp field is not a date".to_string(),
                ))
            }
        };

        let heartrate = fields.iter().find(|&f| f.name() == "heart_rate");
        let heartrate = heartrate
            .map(|hr| hr.clone().into_value())
            .and_then(|hr| match hr {
                Value::UInt8(hr) => Some(i16::from(hr)),
                _ => None,
            });

        let latitude = fields.iter().find(|&f| f.name() == "position_lat");
        let latitude = latitude
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::SInt32(val) => Some(int_to_coord(val)),
                _ => None,
            });
        let longitude = fields.iter().find(|&f| f.name() == "position_long");
        let longitude = longitude
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::SInt32(val) => Some(int_to_coord(val)),
                _ => None,
            });

        let altitude = fields
            .iter()
            .find(|&f| f.name() == "enhanced_altitude" || f.name() == "altitude");
        let altitude = altitude
            .map(|a| a.clone().into_value())
            .and_then(|a| match a {
                Value::Float64(a) => Some(a),
                _ => None,
            });

        let distance = fields
            .iter()
            .find(|&f| f.name() == "enhanced_distance" || f.name() == "distance");
        let distance = distance
            .map(|d| d.clone().into_value())
            .and_then(|d| match d {
                Value::Float64(d) => Some(d),
                _ => None,
            });

        let speed = fields
            .iter()
            .find(|&f| f.name() == "enhanced_speed" || f.name() == "speed");
        let speed = speed.map(|s| s.clone().into_value()).and_then(|s| match s {
            Value::Float64(s) => Some(s),
            _ => None,
        });

        Ok(DatabaseEntry {
            state: Box::new(Record {
                timestamp,
                heartrate,
                latitude,
                longitude,
                altitude,
                distance,
                speed,
            }),
            extra: New,
        })
    }
}
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
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Lap {
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
impl TryFrom<FitDataRecord> for DatabaseEntry<New, Lap> {
    type Error = ModelError;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Lap => {}
            _ => return Err(ModelError::ParseError("Not a Lap".to_string())),
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
            state: Box::new(Lap {
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

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Activity {
    pub user_id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time:DateTime<Local>,
    pub duration: f64,
}
impl TryFrom<FitDataRecord> for DatabaseEntry<New, Activity> {
    type Error = ModelError;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Activity => {}
            _ => return Err(ModelError::ParseError("Not a Activity".to_string())),
        };
        let fields = value.fields();
        let timestamp = fields
            .iter()
            .find(|&f| f.name() == "local_timestamp" || f.name() == "timestamp")
            .ok_or(ModelError::ParseError("no timestamp in record".to_string()))?;
        let timestamp = match timestamp.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => {
                return Err(ModelError::ParseError(
                    "timestamp field is not a date".to_string(),
                ))
            }
        };

        let duration = fields
            .iter()
            .find(|&f| f.name() == "total_timer_time")
            .ok_or(ModelError::ParseError(
                "no total_timer_time in record".to_string(),
            ))?;
        let duration = match duration.clone().into_value() {
            Value::Float64(date) => date,
            _ => {
                return Err(ModelError::ParseError(
                    "duration field is not a date".to_string(),
                ))
            }
        };

        Ok(DatabaseEntry {
            state: Box::new(Activity {
                user_id: None,
                start_time:timestamp,
                end_time:timestamp + Duration::seconds(duration as i64),
                duration,
            }),
            extra: New,
        })
    }
}

fn int_to_coord(value: i32) -> f64 {
    value as f64 / (u64::pow(2, 32) as f64 / 360.0)
}

#[cfg(feature = "ssr")]
pub async fn insert_activity(
    activity: DatabaseEntry<New, Activity>,
    user_id: i64,
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<DatabaseEntry<Stored, Activity>, ModelError> {
    let result = query(
        r#"
        INSERT INTO activities (user_id, start_time, end_time, duration)
        VALUES ($1, $2, $3,$4)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(activity.state.start_time)
    .bind(activity.state.end_time)
    .bind(activity.state.duration)
    .fetch_one(executor)
    .await
    .map_err(|e| ModelError::InsertError(format!("Couldn't insert activity: {}", e)))?;
    let activity_id = result
        .try_get::<i64, _>("id")
        .map_err(|e| ModelError::InsertError(format!("Couldn't get inserted id: {}", e)))?;

    Ok(DatabaseEntry {
        state: activity.state,
        extra: Stored { activity_id },
    })
}

#[cfg(feature = "ssr")]
pub async fn insert_records(
    records: Vec<DatabaseEntry<New, Record>>,
    activity_id: i64,
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<(), ModelError> {
    let num_records = records.len();
    let activity_ids: Vec<i64> = std::iter::repeat(activity_id).take(num_records).collect();
    let (timestamp, heartrate, distance, speed, altitude, latitude, longitude): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = records
        .into_iter()
        .map(|r| {
            (
                r.state.timestamp,
                r.state.heartrate,
                r.state.distance,
                r.state.speed,
                r.state.altitude,
                r.state.latitude,
                r.state.longitude,
            )
        })
        .multiunzip();
    sqlx::query!(
        r#"
        INSERT INTO records(activity_id, timestamp, heartrate, distance, speed, altitude, latitude, longitude)
        SELECT *
        FROM UNNEST($1::bigint[], $2::timestamptz[], $3::smallint[], $4::float8[], $5::float8[], $6::float8[], $7::float8[], $8::float8[])
        "#,
        &activity_ids[..],&timestamp[..], &heartrate[..] as _, &distance[..] as _, &speed[..] as _, &altitude[..] as _, &latitude[..] as _, &longitude[..] as _).execute(executor).await
        .map_err(|e| ModelError::InsertError(format!("Couldn't insert records: {}", e)))?;

    Ok(())
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

#[cfg(feature = "ssr")]
pub async fn insert_laps(
    laps: Vec<DatabaseEntry<New, Lap>>,
    activity_id: i64,
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<(), ModelError> {
    let num_laps = laps.len();
    let activity_ids: Vec<i64> = std::iter::repeat(activity_id).take(num_laps).collect();
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
    ) = laps.clone()
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
    let max_speed: Vec<_> = laps.into_iter().map(|r|r.state.max_speed).collect();
    sqlx::query!(
        r#"
        INSERT INTO laps(activity_id,start_time,end_time,sport,distance,calories,average_heartrate,min_heartrate,max_heartrate,average_power,ascent,descent,average_speed,max_speed)
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
        .map_err(|e| ModelError::InsertError(format!("Couldn't insert lap: {}", e)))?;

    Ok(())
}


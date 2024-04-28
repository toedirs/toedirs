use chrono::{DateTime, Local};
use fitparser::{profile::MesgNum, FitDataRecord, Value};
#[cfg(feature = "ssr")]
use itertools::Itertools;

use super::base::{DatabaseEntry, ModelError, New};

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

fn int_to_coord(value: i32) -> f64 {
    value as f64 / (u64::pow(2, 32) as f64 / 360.0)
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

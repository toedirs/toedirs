use chrono::{DateTime, Duration, Local};
use fitparser::{profile::MesgNum, FitDataRecord, Value};
#[cfg(feature = "ssr")]
use sqlx::{query, Row};

#[cfg(feature = "ssr")]
use super::base::Stored;
use super::base::{DatabaseEntry, ModelError, New};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Activity {
    pub user_id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub duration: f64,
    pub load: Option<u32>,
    pub avg_heartrate: Option<u16>,
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
                start_time: timestamp,
                end_time: timestamp + Duration::try_seconds(duration as i64).unwrap(),
                duration,
                load: None,
                avg_heartrate: None,
            }),
            extra: New,
        })
    }
}

#[cfg(feature = "ssr")]
pub async fn insert_activity(
    activity: DatabaseEntry<New, Activity>,
    user_id: i64,
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<DatabaseEntry<Stored, Activity>, ModelError> {
    let result = query(
        r#"
        INSERT INTO activities (user_id, start_time, end_time, duration,avg_heartrate,load)
        VALUES ($1, $2, $3,$4,$5,$6)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(activity.state.start_time)
    .bind(activity.state.end_time)
    .bind(activity.state.duration)
    .bind(activity.state.avg_heartrate.map(|v| v as i32))
    .bind(activity.state.load.map(|v| v as i32))
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

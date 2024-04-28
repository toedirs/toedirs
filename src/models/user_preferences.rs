use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub user_id: i64,
    pub start_time: Option<DateTime<Local>>,
    pub end_time: Option<DateTime<Local>>,
    pub aerobic_threshold: i32,
    pub anaerobic_threshold: i32,
    pub max_heartrate: i32,
    pub tau: f64,
    pub c: f64,
}

impl UserPreferences {
    /// Calculate training load
    ///
    /// We count how much time in minutes was spent at each heartrate, multiply it by the weighting for that heartrate,
    /// then sum up all the loads to get the total load
    pub fn calculate_load(&self, heartrates: Vec<u32>) -> u32 {
        let hr_buckets = heartrates
            .iter()
            .filter(|&hr| *hr as f64 > self.max_heartrate as f64 * 0.55)
            .fold(HashMap::new(), |mut buckets: HashMap<_, usize>, hr| {
                let count = buckets.entry(hr).or_insert(0);
                *count += 1;
                buckets
            });
        hr_buckets
            .iter()
            .map(|(&hr, time_s)| {
                (self.c * (self.tau * *hr as f64).exp() + 1.0) * *time_s as f64 / 60.0
            })
            .sum::<f64>()
            .round() as u32
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            user_id: 0,
            start_time: None,
            end_time: None,
            aerobic_threshold: 155,
            anaerobic_threshold: 172,
            max_heartrate: 183,
            tau: 0.0809749,
            c: 0.000002370473,
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn get_user_preferences(
    user_id: i64,
    date: DateTime<Local>,
    executor: impl sqlx::PgExecutor<'_>,
) -> UserPreferences {
    let result = sqlx::query_as!(
        UserPreferences,
        r#"
        SELECT
            user_id,
            start_time as "start_time:DateTime<Local>",
            end_time as "end_time:DateTime<Local>",
            aerobic_threshold,
            anaerobic_threshold,
            max_heartrate,
            tau,
            c
        FROM user_preferences
        WHERE user_id=$1 
            and (start_time IS NULL and end_time IS NULL) 
            OR (start_time IS NULL and $2 < end_time) 
            OR (start_time <= $2 and end_time IS NULL) 
            OR (start_time <= $2 and $2 < end_time)
        LIMIT 1
        "#,
        user_id as i32,
        date
    )
    .fetch_optional(executor)
    .await
    .expect("couldn't query user prefences")
    .unwrap_or_default();
    return result;
}

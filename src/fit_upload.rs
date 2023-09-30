use anyhow::{bail, Context, Result};
#[cfg(feature = "ssr")]
use axum::{extract::Multipart, http::StatusCode};
use bytes::Bytes;
use chrono::{DateTime, Local};
use fitparser::{profile::MesgNum, FitDataRecord, Value};
use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_router::*;

#[derive(Debug, Clone)]
struct DatabaseEntry<S: DatabaseState, T> {
    state: Box<T>,
    extra: S,
}

#[derive(Debug, Clone)]
struct New;
#[derive(Debug, Clone)]
struct Inserted {
    activity_id: i64,
}

trait DatabaseState {}
impl DatabaseState for New {}
impl DatabaseState for Inserted {}

#[derive(Debug, Clone)]
struct Coordinates {
    latitude: i32,
    longitude: i32,
}

#[derive(Debug, Clone)]
struct Record {
    timestamp: DateTime<Local>,
    heartrate: Option<u8>,
    coordinates: Option<Coordinates>,
    distance: Option<f64>,
    speed: Option<f64>,
    altitude: Option<f64>,
}

impl TryFrom<FitDataRecord> for DatabaseEntry<New, Record> {
    type Error = &'static str;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Record => {}
            _ => return Err("Not a Record"),
        };
        let fields = value.fields();
        let timestamp = fields
            .iter()
            .find(|&f| f.name() == "timestamp")
            .ok_or("no timestamp in record")?;
        let timestamp = match timestamp.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => return Err("timestamp field is not a date"),
        };

        let heartrate = fields.iter().find(|&f| f.name() == "heart_rate");
        let heartrate = heartrate
            .map(|hr| hr.clone().into_value())
            .and_then(|hr| match hr {
                Value::UInt8(hr) => Some(hr),
                _ => None,
            });

        let latitude = fields.iter().find(|&f| f.name() == "position_lat");
        let longitude = fields.iter().find(|&f| f.name() == "position_long");
        let coordinates = latitude
            .zip(longitude)
            .map(|(lat, long)| (lat.clone().into_value(), long.clone().into_value()))
            .and_then(|(lat, long)| match (lat, long) {
                (Value::SInt32(lat), Value::SInt32(long)) => Some(Coordinates {
                    latitude: lat,
                    longitude: long,
                }),
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
                coordinates,
                altitude,
                distance,
                speed,
            }),
            extra: New,
        })
    }
}
#[derive(Debug, Clone)]
struct Session {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    sport: Option<String>,
    distance: Option<f64>,
    calories: Option<u16>,
    average_heartrate: Option<u8>,
    min_heartrate: Option<u8>,
    max_heartrate: Option<u8>,
    average_power: Option<u16>,
    ascent: Option<u16>,
    descent: Option<u16>,
    average_speed: Option<f64>,
    max_speed: Option<f64>,
}

impl TryFrom<FitDataRecord> for DatabaseEntry<New, Session> {
    type Error = &'static str;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Session => {}
            _ => return Err("Not a Session"),
        };
        let fields = value.fields();
        let start_time = fields
            .iter()
            .find(|&f| f.name() == "start_time")
            .ok_or("no start_time in record")?;
        let start_time = match start_time.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => return Err("start_time field is not a date"),
        };

        let end_time = fields
            .iter()
            .find(|&f| f.name() == "timestamp")
            .ok_or("no end_time in record")?;
        let end_time = match end_time.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => return Err("end_time field is not a date"),
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
                Value::UInt8(val) => Some(val),
                _ => None,
            });

        let min_heartrate = fields.iter().find(|&f| f.name() == "min_heart_rate");
        let min_heartrate = min_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(val),
                _ => None,
            });

        let max_heartrate = fields.iter().find(|&f| f.name() == "max_heart_rate");
        let max_heartrate = max_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(val),
                _ => None,
            });

        let calories = fields
            .iter()
            .find(|&f| f.name() == "calories" || f.name() == "total_calories");
        let calories = calories
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(val),
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
                Value::UInt16(val) => Some(val),
                _ => None,
            });

        let descent = fields
            .iter()
            .find(|&f| f.name() == "descent" || f.name() == "total_descent");
        let descent = descent
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(val),
                _ => None,
            });

        let average_power = fields.iter().find(|&f| f.name() == "avg_power");
        let average_power = average_power
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(val),
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
#[derive(Debug, Clone)]
struct Lap {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    sport: Option<String>,
    distance: Option<f64>,
    calories: Option<u16>,
    average_heartrate: Option<u8>,
    min_heartrate: Option<u8>,
    max_heartrate: Option<u8>,
    average_power: Option<u16>,
    ascent: Option<u16>,
    descent: Option<u16>,
    average_speed: Option<f64>,
    max_speed: Option<f64>,
}
impl TryFrom<FitDataRecord> for DatabaseEntry<New, Lap> {
    type Error = &'static str;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Lap => {}
            _ => return Err("Not a Lap"),
        };
        let fields = value.fields();
        let start_time = fields
            .iter()
            .find(|&f| f.name() == "start_time")
            .ok_or("no start_time in record")?;
        let start_time = match start_time.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => return Err("start_time field is not a date"),
        };

        let end_time = fields
            .iter()
            .find(|&f| f.name() == "timestamp")
            .ok_or("no end_time in record")?;
        let end_time = match end_time.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => return Err("end_time field is not a date"),
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
                Value::UInt8(val) => Some(val),
                _ => None,
            });

        let min_heartrate = fields.iter().find(|&f| f.name() == "min_heart_rate");
        let min_heartrate = min_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(val),
                _ => None,
            });

        let max_heartrate = fields.iter().find(|&f| f.name() == "max_heart_rate");
        let max_heartrate = max_heartrate
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt8(val) => Some(val),
                _ => None,
            });

        let calories = fields
            .iter()
            .find(|&f| f.name() == "calories" || f.name() == "total_calories");
        let calories = calories
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(val),
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
                Value::UInt16(val) => Some(val),
                _ => None,
            });

        let descent = fields
            .iter()
            .find(|&f| f.name() == "descent" || f.name() == "total_descent");
        let descent = descent
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(val),
                _ => None,
            });

        let average_power = fields.iter().find(|&f| f.name() == "avg_power");
        let average_power = average_power
            .map(|val| val.clone().into_value())
            .and_then(|val| match val {
                Value::UInt16(val) => Some(val),
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

#[derive(Debug, Clone)]
struct Activity {
    timestamp: DateTime<Local>,
    duration: f64,
}
impl TryFrom<FitDataRecord> for DatabaseEntry<New, Activity> {
    type Error = &'static str;

    fn try_from(value: FitDataRecord) -> Result<Self, Self::Error> {
        match value.kind() {
            MesgNum::Activity => {}
            _ => return Err("Not a Activity"),
        };
        let fields = value.fields();
        let timestamp = fields
            .iter()
            .find(|&f| f.name() == "local_timestamp" || f.name() == "timestamp")
            .ok_or("no timestamp in record")?;
        let timestamp = match timestamp.clone().into_value() {
            Value::Timestamp(date) => date,
            _ => return Err("timestamp field is not a date"),
        };

        let duration = fields
            .iter()
            .find(|&f| f.name() == "total_timer_time")
            .ok_or("no total_timer_timein record")?;
        let duration = match duration.clone().into_value() {
            Value::Float64(date) => date,
            _ => return Err("duration field is not a date"),
        };

        Ok(DatabaseEntry {
            state: Box::new(Activity {
                timestamp,
                duration,
            }),
            extra: New,
        })
    }
}

#[cfg(feature = "ssr")]
pub async fn upload_fit_file(mut multipart: Multipart) -> StatusCode {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();
        let _ = process_fit_file(data);
    }
    StatusCode::ACCEPTED
}

fn process_fit_file(data: Bytes) -> Result<()> {
    let mut records: Vec<DatabaseEntry<New, Record>> = Vec::new();
    let mut sessions: Vec<DatabaseEntry<New, Session>> = Vec::new();
    let mut laps: Vec<DatabaseEntry<New, Lap>> = Vec::new();
    let mut activity: Option<DatabaseEntry<New, Activity>> = None;
    for data in fitparser::from_bytes(&data).context("Failed to read fit file")? {
        match data.kind() {
            fitparser::profile::MesgNum::Record => {
                DatabaseEntry::<New, Record>::try_from(data).map(|record| records.push(record));
            }
            fitparser::profile::MesgNum::Event => {
                leptos::logging::log!("Event: {:?}", data);
            }
            fitparser::profile::MesgNum::Session => {
                DatabaseEntry::<New, Session>::try_from(data).map(|session| sessions.push(session));
            }
            fitparser::profile::MesgNum::Lap => {
                DatabaseEntry::<New, Lap>::try_from(data).map(|lap| laps.push(lap));
            }
            fitparser::profile::MesgNum::Activity => {
                if let Some(_) = activity {
                    bail!("Found more than one activity");
                }
                activity = Some(
                    DatabaseEntry::<New, Activity>::try_from(data)
                        .expect("no activity entry found"),
                );
            }
            fitparser::profile::MesgNum::DeviceInfo => {
                leptos::logging::log!("Device Info: {:?}", data);
            }
            _ => {
                leptos::logging::log!("Unknown: {:?}", data.kind())
            }
        }
    }
    if let None = activity {
        bail!("No activity found in fit file, may be corrupt");
    }
    // leptos::logging::log!("Parsed records: {:?}", records);
    Ok(())
}

#[component]
pub fn FitUploadForm(show: ReadSignal<bool>, show_set: WriteSignal<bool>) -> impl IntoView {
    let on_submit = move |_ev: SubmitEvent| {
        show_set(false);
        println!("hidden?")
    };
    leptos::view! {
        <Show when=move || { show() } fallback=|| { () }>
            <div
                class="modal bottom-sheet"
                style="z-index: 1003; display: block; opacity: 1; bottom: 0%"
            >
                <Form
                    action="/api/upload_fit_file"
                    method="POST"
                    enctype="multipart/form-data".to_string()
                    on:submit=on_submit
                >
                    <div
                        class="modal-content">
                        <h4 class="black-text">"Upload Fit File"</h4>
                        <div class="row">
                            <div class="col s12 input-field file-field">
                                <div class="btn">
                                    <span>File</span>
                                    <input type="file" name="fit_file" multiple/>
                                </div>
                                <div class="file-path-wrapper">
                                    <input class="file-path validate" type="text"/>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="submit" class="btn waves-effect waves-light">
                            <i class="material-symbols-rounded right">upload</i>
                            Upload
                        </button>
                    </div>
                </Form>
            </div>
            <div class="modal-overlay" style="z-index: 1002; display: block; opacity: 0.5;" on:click=move |_|{show_set(false)}></div>
        </Show>
    }
}

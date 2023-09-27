use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_router::*;

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

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
        use axum::extract::Multipart;
        use bytes::Bytes;
        use fitparser::{FitDataRecord, Value, profile::MesgNum};

        impl TryFrom<FitDataRecord> for Record {
            type Error = &'static str;

            fn try_from(value: FitDataRecord)-> Result<Self, Self::Error> {
                match value.kind() {
                    MesgNum::Record => {},
                    _ => return Err("Not a Record")
                };
                let fields = value.fields();
                let timestamp = fields.iter().find(|&f| f.name() == "timestamp").ok_or("no timestamp in record")?;
                let timestamp = match timestamp.clone().into_value() {
                    Value::Timestamp(date) => date,
                    _ => return Err("timestamp field is not a date")
                };

                let heartrate = fields.iter().find(|&f| f.name() == "heart_rate");
                let heartrate = heartrate.map(|hr| hr.clone().into_value()).and_then(|hr| match hr {Value::UInt8(hr)=> Some(hr), _ => None});


                let latitude = fields.iter().find(|&f| f.name() == "position_lat");
                let longitude = fields.iter().find(|&f| f.name() == "position_long");
                let coordinates = latitude.zip(longitude).map(|(lat, long)| (lat.clone().into_value(), long.clone().into_value())).and_then(|(lat, long)| match (lat,long) { (Value::SInt32(lat), Value::SInt32(long)) => Some(Coordinates {latitude: lat, longitude: long}), _ => None});

                let altitude = fields.iter().find(|&f|f.name() == "enhanced_altitude" || f.name() == "altitude");
                let altitude = altitude.map(|a| a.clone().into_value()).and_then(|a| match a { Value::Float64(a) => Some(a), _ => None});

                let distance = fields.iter().find(|&f|f.name() == "enhanced_distance" || f.name() == "distance");
                let distance = distance.map(|d| d.clone().into_value()).and_then(|d| match d { Value::Float64(d) => Some(d), _ => None});

                let speed = fields.iter().find(|&f|f.name() == "enhanced_speed" || f.name() == "speed");
                let speed = speed.map(|s| s.clone().into_value()).and_then(|s| match s { Value::Float64(s) => Some(s), _ => None});

                Ok(Record {timestamp, heartrate, coordinates, altitude, distance, speed})
            }
        }

        pub async fn upload_fit_file(mut multipart: Multipart) -> axum::http::StatusCode {
            while let Some(field) = multipart.next_field().await.unwrap() {
                let data = field.bytes().await.unwrap();
                let _ = process_fit_file(data);
            }
            axum::http::StatusCode::ACCEPTED
        }

        fn process_fit_file(data: Bytes) -> Result<()> {
            let mut records: Vec<Record> = Vec::new();
            for data in fitparser::from_bytes(&data).context("Failed to read fit file")? {
                match data.kind() {
                    fitparser::profile::MesgNum::Record => {
                        Record::try_from(data).map(|record| records.push(record));
                    },
                    fitparser::profile::MesgNum::Event => {leptos::logging::log!("Event: {:?}", data);},
                    fitparser::profile::MesgNum::Session => {leptos::logging::log!("Session: {:?}", data);},
                    fitparser::profile::MesgNum::Lap => {leptos::logging::log!("Lap: {:?}", data);},
                    fitparser::profile::MesgNum::Activity => {leptos::logging::log!("Activity: {:?}", data);},
                    fitparser::profile::MesgNum::DeviceInfo => {leptos::logging::log!("Device Info: {:?}", data);},
                    _ => {leptos::logging::log!("Unknown: {:?}", data.kind())}
                }
            }
            leptos::logging::log!("Parsed records: {:?}", records);
            Ok(())
        }
    }
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
                            <input type="file" name="fit_file" multiple/>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="submit" class="btn waves-effect waves-light">
                            Upload
                            <i class="material-symbols-rounded right">upload</i>
                        </button>
                    </div>
                </Form>
            </div>
            <div class="modal-overlay" style="z-index: 1002; display: block; opacity: 0.5;"></div>
        </Show>
    }
}

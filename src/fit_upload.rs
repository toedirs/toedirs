use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_router::*;

#[derive(Debug, Clone)]
struct Record {
    timestamp: DateTime<Local>,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
        use axum::extract::Multipart;
        use bytes::Bytes;
        use leptos::logging::log;
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
                Ok(Record {timestamp})
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
                    fitparser::profile::MesgNum::Record => {Record::try_from(data).map(|record| records.push(record));},
                    fitparser::profile::MesgNum::Event => {leptos::logging::log!("Event: {:?}", data);},
                    fitparser::profile::MesgNum::Session => {leptos::logging::log!("Session: {:?}", data);},
                    fitparser::profile::MesgNum::Lap => {leptos::logging::log!("Lap: {:?}", data);},
                    fitparser::profile::MesgNum::Activity => {leptos::logging::log!("Activity: {:?}", data);},
                    fitparser::profile::MesgNum::DeviceInfo => {leptos::logging::log!("Device Info: {:?}", data);},
                    _ => {leptos::logging::log!("Unknown: {:?}", data.kind())}
                }
            }
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

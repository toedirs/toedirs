use anyhow::{Context, Result};
#[cfg(feature = "ssr")]
use axum::extract::Multipart;
use bytes::Bytes;
#[cfg(feature = "ssr")]
use futures_util::stream::StreamExt;
use leptos::logging::log;
use leptos::{ev::SubmitEvent, *};
use leptos_router::*;

// #[server(FitUpload, "/api")]
// pub async fn upload_fit_file() -> Result<String, ServerFnError> {
//     use axum::{
//         extract::{Field, Multipart},
//         http::Method,
//     };
//     use leptos_axum::extract;

//     extract(|method: Method, multipart: Multipart| async move {
//         while let Some(mut field) = multipart.next_field().await.unwrap() {
//             let name = field.name().unwrap().to_string();
//             process_fit_file(field.bytes().await.unwrap());
//         }
//     })
//     .await
//     .map_err(|e| ServerFnError::ServerError("Couldn't extract multipart".to_string()));
// }

#[cfg(feature = "ssr")]
pub async fn upload_fit_file(mut multipart: Multipart) -> axum::http::StatusCode {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();
        let _ = process_fit_file(data);
    }
    axum::http::StatusCode::ACCEPTED
}

fn process_fit_file(data: Bytes) -> Result<()> {
    for data in fitparser::from_bytes(&data).context("Failed to read fit file")? {
        log!("{:?}", data);
    }
    Ok(())
}

#[component]
pub fn FitUploadForm(show: ReadSignal<bool>, show_set: WriteSignal<bool>) -> impl IntoView {
    let on_submit = move |ev: SubmitEvent| {
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

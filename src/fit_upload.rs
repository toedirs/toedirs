use anyhow::{Context, Result};
use bytes::Bytes;
use leptos::ev::SubmitEvent;
use leptos::logging::log;
use leptos::*;
use leptos_router::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
use axum::extract::Multipart;
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
}}

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

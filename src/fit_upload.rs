use anyhow::{Context, Result};
#[cfg(feature = "ssr")]
use axum::extract::Multipart;
use bytes::Bytes;
#[cfg(feature = "ssr")]
use futures_util::stream::StreamExt;
use leptos::*;
use leptos_router::*;

// #[server(FitUpload, "/api")]
// pub async fn upload_fit_file(cx: Scope) -> Result<String, ServerFnError> {
//     use axum::{
//         extract::{Field, Multipart},
//         http::Method,
//     };
//     use leptos_axum::extract;

//     extract(cx, |method: Method, multipart: Multipart| async move {
//         while let Some(mut field) = multipart.next_field().await.unwrap() {
//             let name = field.name().unwrap().to_string();
//             process_fit_file(field.bytes().await.unwrap());
//         }
//     })
//     .await
//     .map_err(|e| ServerFnError::ServerError("Couldn't extract multipart".to_string()));
// }

#[cfg(feature = "ssr")]
async fn upload_fit_file(mut multipart: Multipart) {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();
        process_fit_file(data);
    }
}

fn process_fit_file(data: Bytes) -> Result<()> {
    for data in fitparser::from_bytes(&data).context("Failed to read fit file")? {
        log!("{:?}", data);
    }
    Ok(())
}

#[component]
pub fn FitUploadForm(cx: Scope, show_upload_modal: ReadSignal<bool>) -> impl IntoView {
    let upload_fit_file = create_server_action::<FitUpload>(cx);
    view! { cx,
        <Show when=move || { show_upload_modal() } fallback=|_| { () }>
            <div class="relative z-10" role="dialog" aria-modal="true">
                <Form action="/api/upload_fit_file">
                    <label>"Upload Fit File"</label>
                    <input type="file" name="fit_file" multiple/>
                    <input type="submit" value="Upload"/>
                </Form>
            </div>
        </Show>
    }
}

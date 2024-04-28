use crate::app::App;
use axum::response::Response as AxumResponse;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
};
use leptos::*;
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let encoding = req
        .headers()
        .get("accept-encoding")
        .map(|h| h.to_str().unwrap_or("none"))
        .unwrap_or("none");
    let res = get_static_file(uri.clone(), &root, encoding).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
        handler(req).await.into_response()
    }
}

async fn get_static_file(
    uri: Uri,
    root: &str,
    encoding: &str,
) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .header("Accept-Encoding", encoding)
        .body(Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root)
        .precompressed_gzip()
        .precompressed_br()
        .oneshot(req)
        .await
    {
        Ok(res) => Ok(res.into_response()),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}

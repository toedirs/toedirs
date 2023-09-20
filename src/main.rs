mod fit_upload;

#[cfg(feature = "ssr")]
use sqlx::postgres::{PgPoolOptions, Pool, PgPool};
#[cfg(feature = "ssr")]
static CONNECTION_POOL: OnceCell<Pool<PgPool> = OnceCell::new();

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::post, Router};
    use leptos::logging::log;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use toedirs::app::*;
    use toedirs::config::Config;
    use toedirs::fileserv::file_and_error_handler;
    use once_cell::sync::OnceCell;
    

    Config::load();
    
    let pool = PgPoolOptions::new().max_connections(50).connect(format!("postgresql://{credentials}{host}/{database}", ))
    CONNECTION_POOL.set()

    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved t:wao deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|| view! { <App/> }).await;

    // build our application with a route
    let app = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/api/upload_fit_file", post(fit_upload::upload_fit_file))
        .leptos_routes(&leptos_options, routes, || view! { <App/> })
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

impl Pool<PgPool> { 
    pub fn global() -> &'static Pool<PgPool> {
        CONNECTION_POOL.get().expect("connection pool not initialized")
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

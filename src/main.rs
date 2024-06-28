use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{routing::{get,post}, Router, response::{Response,IntoResponse}, extract::{Path, State }, http::{Request }, body::Body as AxumBody};
        use leptos::logging::log;
        use leptos::*;
        use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
        use toedirs::app::*;
        use toedirs::authentication::*;
        use toedirs::pages::fit_upload::upload_fit_file;
        use toedirs::state::AppState;
        use toedirs::config::Config;
        use toedirs::fileserv::file_and_error_handler;
        use sqlx::{PgPool,ConnectOptions, migrate, {postgres::{PgPoolOptions,PgConnectOptions}}};

        use axum_session::{SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::{AuthSessionLayer, AuthConfig, SessionPgPool};
        use sentry::integrations::tower::{NewSentryLayer};
    }
}

#[cfg(feature = "ssr")]
async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    _path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
        },
        request,
    )
    .await
}

#[cfg(feature = "ssr")]
async fn leptos_routes_handler(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
        },
        App,
    );
    handler(req).await.into_response()
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use log::LevelFilter;

    Config::load();
    let config = Config::global();
    log!("connecting to db: {}", config.database_url);
    let db_connect_options: PgConnectOptions = config
        .database_url
        .parse::<PgConnectOptions>()
        .expect("unable to parse database url")
        .log_statements(LevelFilter::Info);

    let pool = PgPoolOptions::new()
        .max_connections(50)
        // .connect(config.database_url.as_str())
        .connect_with(db_connect_options)
        .await
        .expect("couldn't connect to database");

    let session_config = SessionConfig::default().with_table_name("axum_sessions");
    let auth_config = AuthConfig::<i64>::default();
    let session_store =
        SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config)
            .await
            .expect("couldn't create session store");
    migrate!().run(&pool).await.expect("migrations to run");

    simple_logger::init_with_level(log::Level::Warn).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved t:wao deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let app_state = AppState {
        leptos_options,
        pool: pool.clone(),
        routes: routes.clone(),
    };

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route("/api/upload_fit_file", post(upload_fit_file))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .layer(NewSentryLayer::new_from_top())
        .layer(
            AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(Some(pool.clone()))
                .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

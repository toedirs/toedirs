mod fit_upload;
use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{routing::{get,post}, Router, response::{Response,IntoResponse}, extract::{Path, State, RawQuery}, http::{Request, header::HeaderMap}, body::Body as AxumBody};
        use leptos::logging::log;
        use leptos::*;
        use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
        use toedirs::app::*;
        use toedirs::auth::*;
        use toedirs::state::AppState;
        use toedirs::config::Config;
        use toedirs::fileserv::file_and_error_handler;
        use sqlx::{PgPool, migrate, {postgres::{PgPoolOptions}}};

        use axum_session::{SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::{AuthSessionLayer, AuthConfig, SessionPgPool};
        async fn server_fn_handler(State(app_state): State<AppState>, auth_session: AuthSession, path: Path<String>, headers: HeaderMap, raw_query: RawQuery,
            request: Request<AxumBody>) -> impl IntoResponse {

            log!("{:?}", path);

            handle_server_fns_with_context(path, headers, raw_query, move || {
                provide_context(auth_session.clone());
                provide_context(app_state.pool.clone());
            }, request).await
        }
        async fn leptos_routes_handler(auth_session: AuthSession, State(app_state): State<AppState>, req: Request<AxumBody>) -> Response{
            let handler = leptos_axum::render_app_to_stream_with_context(app_state.leptos_options.clone(),
                move || {
                    provide_context(auth_session.clone());
                    provide_context(app_state.pool.clone());
                },
                || view! {<App/> }
            );
            handler(req).await.into_response()
        }

        #[tokio::main]
        async fn main() {
            Config::load();
            let config = Config::global();

            let pool = PgPoolOptions::new().max_connections(50).connect(format!("postgresql://{user}:{password}@{host}:{port}/{database}", user=config.db.user, password=config.db.password, host=config.db.host, port=config.db.port, database=config.db.database).as_str()).await.expect("couldn't connect to database");

            let session_config = SessionConfig::default().with_table_name("axum_sessions");
            let auth_config = AuthConfig::<i64>::default();
            let session_store = SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config).await.expect("couldn't create session store");
            migrate!().run(&pool).await.expect("migrations to run");

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
            let app_state = AppState{
                leptos_options,
                pool: pool.clone(),
                routes: routes.clone()
            };

            // build our application with a route
            let app = Router::new()
                .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
                .route("/api/upload_fit_file", post(fit_upload::upload_fit_file))
                .leptos_routes_with_handler( routes, get(leptos_routes_handler))
                .fallback(file_and_error_handler)
                .layer(AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(Some(pool.clone()))
                .with_config(auth_config))
                .layer(SessionLayer::new(session_store))
                .with_state(app_state);

            // run our app with hyper
            // `axum::Server` is a re-export of `hyper::Server`
            log!("listening on http://{}", &addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }

    }
}
#[cfg(feature = "ssr")]
#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

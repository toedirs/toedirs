use crate::{
    auth,
    error_template::{AppError, ErrorTemplate},
    pages::{
        activity_overview::ActivityList,
        auth::{login::Login, signup::Signup},
        fit_upload::FitUploadForm,
        home::Home,
        landing::Landing,
        overview::Overview,
        user::UserSettings,
        workout_schedule::WorkoutCalendar,
    },
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {
        use sqlx::PgPool;

        pub fn pool() -> Result<PgPool, ServerFnError> {
           use_context::<PgPool>()
                .ok_or_else(|| ServerFnError::new("Pool missing."))
        }

        pub fn auth() -> Result<auth::AuthSession, ServerFnError> {
            use_context::<auth::AuthSession>()
                .ok_or_else(|| ServerFnError::new("Auth session missing."))
        }
    }
}
#[derive(Copy, Clone)]
pub struct FitFileUploaded(pub RwSignal<i32>);

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    let uploaded = create_rw_signal(0);
    provide_context(FitFileUploaded(uploaded));
    let (show_upload, set_show_upload) = create_signal(false);
    let show_settings = create_rw_signal(false);
    let login = create_server_action::<auth::Login>();
    let logout = create_server_action::<auth::Logout>();
    let signup = create_server_action::<auth::Signup>();
    let user = create_blocking_resource(
        move || {
            (
                login.version().get(),
                logout.version().get(),
                signup.version().get(),
            )
        },
        move |_| async move {
            let user = auth::get_user().await.unwrap_or(None);
            user.is_some()
        },
    );
    provide_meta_context();
    view! {
        <Stylesheet id="leaflet" href="/leaflet/leaflet.css"/>
        <Script id="leafletjs" src="/leaflet/leaflet.js"/>
        <Script src="https://cdn.jsdelivr.net/npm/echarts@5.4.2/dist/echarts.min.js"/>
        <Script src="https://cdn.jsdelivr.net/npm/echarts-gl@2.0.9/dist/echarts-gl.min.js"/>
        <Stylesheet id="leptos" href="/pkg/toedirs.css"/>

        // sets the document title
        <Title text="Welcome to Toedi"/>
        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <Routes>
                <Route
                    path="/home"
                    view=move || {
                        view! {
                            <Suspense fallback=|| ()>
                                {move || match user.get() {
                                    Some(false) => view! { <Home/> }.into_view(),
                                    Some(true) => view! { <Redirect path="/"/> }.into_view(),
                                    None => ().into_view(),
                                }}

                            </Suspense>
                        }
                    }
                >

                    <Route path="/signup" view=move || view! { <Signup action=signup/> }/>
                    <Route path="/login" view=move || view! { <Login action=login/> }/>
                    <Route path="" view=Landing/>
                </Route>
                <Route
                    path="/"
                    view=move || {
                        view! {
                            <Suspense fallback=|| ()>
                                {move || match user.get() {
                                    Some(true) => {
                                        view! {
                                            <nav
                                                class="navbar is-black"
                                                role="navigation"
                                                aria-label="main navigation"
                                            >
                                                <div class="navbar-brand">
                                                    <a href="/" class="navbar-item">
                                                        Toedi
                                                    </a>
                                                </div>
                                                <div class="navbar-menu">
                                                    <div class="navbar-start">

                                                        <A href="/" class="navbar-item">
                                                            Overview
                                                        </A>

                                                        <A href="/activities" class="navbar-item">
                                                            Activities
                                                        </A>

                                                        <A href="/calendar" class="navbar-item">
                                                            Calendar
                                                        </A>
                                                        <a
                                                            href="#"
                                                            class="navbar-item"
                                                            on:click=move |_| { show_settings.update(|v| *v = !*v) }
                                                        >
                                                            Settings
                                                        </a>
                                                    </div>
                                                    <div class="navbar-end">
                                                        <div class="navbar-item">
                                                            <div class="buttons">
                                                                <a
                                                                    class="button is-primary"
                                                                    on:click=move |_| { set_show_upload.update(|v| *v = !*v) }
                                                                >
                                                                    Upload
                                                                    <i class="material-symbols-rounded right">upload</i>
                                                                </a>
                                                                <ActionForm action=logout>
                                                                    <button type="submit" class="button">
                                                                        "Log Out"
                                                                    </button>
                                                                </ActionForm>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </nav>
                                            <main>
                                                <Outlet/>
                                                <FitUploadForm show=show_upload show_set=set_show_upload/>
                                                <UserSettings show=show_settings/>
                                            </main>
                                        }
                                            .into_view()
                                    }
                                    Some(false) => view! { <Redirect path="/home"/> }.into_view(),
                                    None => ().into_view(),
                                }}

                            </Suspense>
                        }
                    }
                >

                    <Route path="" view=Overview/>

                    <Route path="/activities" view=ActivityList/>
                    <Route path="/calendar" view=WorkoutCalendar/>

                </Route>
            </Routes>
        </Router>
    }
}

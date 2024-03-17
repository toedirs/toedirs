use crate::{
    activity_overview::ActivityList,
    auth::*,
    error_template::{AppError, ErrorTemplate},
    fit_upload::FitUploadForm,
    heartrate_summary_chart::HeartrateSummaryChart,
    training_load_chart::TrainingLoadChart,
    workout_schedule::WorkoutCalendar,
};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, TimeZone};
use leptos::{html::Label, *};
use leptos_meta::*;
use leptos_router::*;

cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {
        use sqlx::PgPool;

        pub fn pool() -> Result<PgPool, ServerFnError> {
           use_context::<PgPool>()
                .ok_or_else(|| ServerFnError::new("Pool missing."))
        }

        pub fn auth() -> Result<AuthSession, ServerFnError> {
            use_context::<AuthSession>()
                .ok_or_else(|| ServerFnError::new("Auth session missing."))
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    let (show_upload, set_show_upload) = create_signal(false);
    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();
    let user = create_blocking_resource(
        move || {
            (
                login.version().get(),
                logout.version().get(),
                signup.version().get(),
            )
        },
        move |_| async move {
            let user = get_user().await.unwrap_or(None);
            user.is_some()
        },
    );
    provide_meta_context();
    view! {
        <Stylesheet id="leaflet" href="/leaflet/leaflet.css"/>
        <Script id="leafletjs" src="/leaflet/leaflet.js"/>
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

#[component]
fn Overview() -> impl IntoView {
    //overview page
    let from_date = create_rw_signal(Some(
        (Local::now() - Duration::days(120))
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    ));
    let to_date = create_rw_signal(Some(
        Local::now().date_naive().format("%Y-%m-%d").to_string(),
    ));
    let from_memo = create_memo(move |_| {
        from_date().and_then(|d| {
            NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                .map(|d| {
                    Local
                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                        .unwrap()
                })
                .ok()
        })
    });
    let to_memo = create_memo(move |_| {
        to_date().and_then(|d| {
            NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                .map(|d| {
                    Local
                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                        .unwrap()
                })
                .ok()
        })
    });
    view! {
        <div class="container">
            <div class="columns">
                <div class="column">

                    <div class="field">
                        <label for="from_date">From</label>
                        <div class="control">
                            <input
                                class="input"
                                type="date"
                                value=from_date
                                on:change=move |ev| {
                                    from_date
                                        .update(|v| {
                                            *v = Some(event_target_value(&ev));
                                        })
                                }
                            />

                        </div>
                    </div>
                </div>
                <div class="column">
                    <div class="field">
                        <label for="to_date">To</label>
                        <div class="control">
                            <input
                                class="input"
                                type="date"
                                value=to_date
                                on:change=move |ev| {
                                    to_date
                                        .update(|v| {
                                            *v = Some(event_target_value(&ev));
                                        })
                                }
                            />

                        </div>
                    </div>

                </div>
            </div>
            <div class="columns is-variable is-1">
                <div class="column is-flex">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Hearrate Zones</p>
                        </div>
                        <div class="card-content">
                            <div class="content">
                                <HeartrateSummaryChart from=from_memo to=to_memo/>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="column is-flex">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Training LoadChart</p>
                        </div>
                        <div class="card-content ">
                            <TrainingLoadChart from=from_memo to=to_memo/>
                        </div>
                    </div>
                </div>
                <div class="column is-flex">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Fitness & Fatigue</p>
                        </div>
                        <div class="card-content">
                            <div class="content">
                                <div class="block">test</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Login(action: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1 class="title">"Log In"</h1>
            <div class="field">
                <label for="username">"User ID:"</label>
                <div class="control">
                    <input
                        class="input"
                        type="text"
                        placeholder="User ID"
                        maxlength="32"
                        name="username"
                        id="username"
                    />

                </div>
            </div>
            <div class="field">
                <label for="password">"Password:"</label>
                <div class="control">
                    <input class="input" type="password" placeholder="Password" name="password"/>

                </div>
            </div>
            <div class="field">
                <div class="control">
                    <label>
                        <input type="checkbox" name="remember"/>
                        "Remember me?"
                    </label>
                </div>
            </div>
            <div class="field is-grouped">
                <p class="control">
                    <button type="submit" class="button is-primary">
                        "Log In"
                    </button>
                </p>
                <p class="control">
                    <A href="/home/signup" class="button">
                        Signup
                    </A>
                </p>
            </div>
        </ActionForm>
    }
}

#[component]
fn Signup(action: Action<Signup, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1 class="title">"Sign Up"</h1>
            <div class="field">
                <label for="username">"User ID:"</label>
                <div class="control">
                    <input
                        class="input"
                        type="text"
                        placeholder="User ID"
                        maxlength="32"
                        name="username"
                        id="username"
                    />

                </div>
            </div>
            <div class="field">
                <label for="password">"Password:"</label>
                <div class="control">
                    <input
                        class="input"
                        type="password"
                        placeholder="Password"
                        name="password"
                        id="password"
                    />

                </div>
            </div>
            <div class="field">
                <label for="password_confirmation">"Confirm Password:"</label>
                <div class="control">
                    <input
                        class="input"
                        type="password"
                        placeholder="Password again"
                        name="password_confirmation"
                        id="password_confirmation"
                    />

                </div>
            </div>
            <div class="field">
                <div class="control">
                    <label>
                        <input type="checkbox" name="remember"/>
                        "Remember me?"
                    </label>
                </div>
            </div>

            <div class="field is-grouped">
                <p class="control">
                    <button type="submit" class="button is-primary">
                        "Sign Up"
                    </button>
                </p>
                <p class="control">

                    <A href="/home/login" class="button">
                        Login
                    </A>
                </p>
            </div>
        </ActionForm>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <nav class="navbar is-black" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <a href="#" class="navbar-item">
                    Toedi
                </a>
            </div>
            <div class="navbar-end">
                <div class="buttons">
                    <A href="/home/login" exact=true class="button is-primary">
                        Login
                    </A>
                    <A href="/home/signup" exact=true class="button">
                        Signup
                    </A>
                </div>
            </div>
        </nav>
        <main>
            <div class="container">
                <Outlet/>
            </div>
        </main>
    }
}
#[component]
fn Landing() -> impl IntoView {
    view! {
        <div class="container">
            <h1 class="title">Welcome to Toedi</h1>
            <div class="columns">
                <div class="column is-flex">
                    <div class="card">
                        <div class="card-header">
                            <p class="card-header-title">Track your Training</p>
                        </div>
                        <div class="card-content">
                            <p>
                                Always stay on top of your training effort with easy to read charts and metrics
                            </p>
                        </div>
                    </div>
                </div>
                <div class="column is-flex">

                    <div class="card">
                        <div class="card-header">
                            <p class="card-header-title">Based on Science</p>
                        </div>
                        <div class="card-content">
                            <p>
                                "Based on newest scientific research, presented in a transparent way. We don't just make up numbers and we explain exactly how our metrics are calculated"
                            </p>
                        </div>
                    </div>
                </div>
                <div class="column is-flex">

                    <div class="card">
                        <div class="card-header">
                            <p class="card-header-title">Open Source</p>
                        </div>
                        <div class="card-content">
                            <p>Fully Open-Source code, made by users for users</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

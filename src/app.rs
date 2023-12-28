use crate::{
    activity_list::ActivityList,
    auth::*,
    error_template::{AppError, ErrorTemplate},
    fit_upload::FitUploadForm,
    heartrate_summary_chart::HeartrateSummaryChart,
    training_load_chart::TrainingLoadChart,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {
        use sqlx::PgPool;

        pub fn pool() -> Result<PgPool, ServerFnError> {
           use_context::<PgPool>()
                .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
        }

        pub fn auth() -> Result<AuthSession, ServerFnError> {
            use_context::<AuthSession>()
                .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
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
        <Stylesheet id="leptos" href="/pkg/toedirs.css"/>

        // sets the document title
        <Title text="Welcome to Toedi"/>

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
                                            <nav class="teal lighten-2">
                                                <div class="nav-wrapper">
                                                    <a href="#" class="brand-logo">
                                                        Toedi
                                                    </a>
                                                    <ul id="nav-mobile" class="right hide-on-med-and-down">
                                                        <li>

                                                            <A href="/" class="">
                                                                Overview
                                                            </A>
                                                        </li>
                                                        <li>

                                                            <A href="/activities" class="">
                                                                Activities
                                                            </A>
                                                        </li>
                                                        <li>
                                                            <a
                                                                class="waves-effect waves-light btn"
                                                                on:click=move |_| { set_show_upload.update(|v| *v = !*v) }
                                                            >
                                                                Upload
                                                                <i class="material-symbols-rounded right">upload</i>
                                                            </a>
                                                        </li>
                                                        <li>
                                                            <ActionForm action=logout>
                                                                <button type="submit" class="btn-flat waves-effect">
                                                                    "Log Out"
                                                                </button>
                                                            </ActionForm>
                                                        </li>
                                                    </ul>
                                                </div>
                                            </nav>
                                            <main>
                                                <div class="container">
                                                    <Outlet/>
                                                    <FitUploadForm show=show_upload show_set=set_show_upload/>
                                                </div>
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

                </Route>
            </Routes>
        </Router>
    }
}

#[component]
fn Overview() -> impl IntoView {
    //overview page
    view! {
        <div class="row">
            <div class="col s12 m6 l4 p-1">
                <div class="card">
                    <div class="card-content teal white-text">
                        <span class="card-title">Hearrate Zones</span>
                        <HeartrateSummaryChart/>
                    </div>
                </div>
            </div>
            <div class="col s12 m6 l4 p-1">
                <div class="card">
                    <div class="card-content teal white-text">
                        <span class="card-title">Training LoadChart</span>
                        <TrainingLoadChart/>
                    </div>
                </div>
            </div>
            <div class="col s12 m6 l4 p-1">
                <div class="card-panel teal">Fitness & Fatigue</div>
            </div>
        </div>
    }
}

#[component]
fn Login(action: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <div class="row">
                <div class="col s12">
                    <h1>"Log In"</h1>
                </div>
            </div>
            <div class="row">
                <div class="input-field col s12">
                    <input
                        type="text"
                        placeholder="User ID"
                        maxlength="32"
                        name="username"
                        id="username"
                    />
                    <label for="username">"User ID:"</label>
                </div>
            </div>
            <div class="row">
                <div class="input-field col s12">
                    <input type="password" placeholder="Password" name="password"/>
                    <label for="password">"Password:"</label>
                </div>
            </div>
            <div class="row">
                <div class="col s12">
                    <label>
                        <input type="checkbox" name="remember"/>
                        <span>"Remember me?"</span>
                    </label>
                </div>
            </div>
            <div class="row">
                <div class="col s12">
                    <button type="submit" class="btn waves-effect waves-light">
                        "Log In"
                    </button>
                    <A href="/home/signup" class="waves-effect waves-light grey-darken-2 btn">
                        Signup
                    </A>
                </div>
            </div>
        </ActionForm>
    }
}

#[component]
fn Signup(action: Action<Signup, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <div class="row">
                <div class="col s12">
                    <h1>"Sign Up"</h1>
                </div>
            </div>
            <div class="row">
                <div class="input-field col s12">
                    <input
                        type="text"
                        placeholder="User ID"
                        maxlength="32"
                        name="username"
                        id="username"
                    />
                    <label for="username">"User ID:"</label>
                </div>
            </div>
            <div class="row">
                <div class="input-field col s12">
                    <input type="password" placeholder="Password" name="password" id="password"/>
                    <label for="password">"Password:"</label>
                </div>
            </div>
            <div class="row">
                <div class="input-field col s12">
                    <input
                        type="password"
                        placeholder="Password again"
                        name="password_confirmation"
                        id="password_confirmation"
                    />
                    <label for="password_confirmation">"Confirm Password:"</label>
                </div>
            </div>
            <div class="row">
                <div class="col s12">
                    <label>
                        <input type="checkbox" name="remember"/>
                        <span>"Remember me?"</span>
                    </label>
                </div>
            </div>

            <div class="row">
                <div class="col s12">
                    <button type="submit" class="btn waves-effect waves-light">
                        "Sign Up"
                    </button>
                    <A href="/home/login" class="btn waves-effect waves-light grey-darken-2">
                        Login
                    </A>
                </div>
            </div>
        </ActionForm>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <nav class="teal lighten-2">
            <div class="nav-wrapper">
                <a href="#" class="brand-logo">
                    Toedi
                </a>
                <ul id="nav-mobile" class="right hide-on-med-and-down">
                    <li>
                        <A href="/home/login" exact=true>
                            Login
                        </A>
                    </li>
                    <li>
                        <A href="/home/signup" exact=true>
                            Signup
                        </A>
                    </li>
                </ul>
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
        <div class="row">
            <div class="col s12">
                <h1>Welcome to Toedi</h1>
            </div>
        </div>
        <div class="row">
            <div class="col s4">
                <div class="card m-1 small">
                    <div class="card-content">
                        <span class="card-title">Track your Training</span>
                        <p>
                            Always stay on top of your training effort with easy to read charts and metrics
                        </p>
                    </div>
                </div>
            </div>
            <div class="col s4">

                <div class="card m-1 small">
                    <div class="card-content">
                        <span class="card-title">Based on Science</span>
                        <p>
                            "Based on newest scientific research, presented in a transparent way. We don't just make up numbers and we explain exactly how our metrics are calculated"
                        </p>
                    </div>
                </div>
            </div>
            <div class="col s4">

                <div class="card m-1 small">
                    <div class="card-content">
                        <span class="card-title">Open Source</span>
                        <p>Fully Open-Source code, made by users for users</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

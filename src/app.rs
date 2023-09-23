use crate::{
    auth::*,
    error_template::{AppError, ErrorTemplate},
    fit_upload::FitUploadForm,
};
use leptos::logging::log;
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
    let (login_status, set_login_status) = create_signal(false);

    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();

    let user = create_blocking_resource(
        move || {
            (
                login.version().get(),
                signup.version().get(),
                logout.version().get(),
            )
        },
        move |_| async move {
            let user = get_user().await;
            {
                let x = if let Ok(ref x) = user {
                    x.is_some()
                } else {
                    false
                };
                log!("login status: {:?}", x);
                set_login_status(x);
            }
            user
        },
    );
    provide_meta_context();
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/toedirs.css"/>

        // sets the document title
        <Title text="Welcome to Toedi"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
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
                        <Transition fallback=move || view! {<span>Loading...</span>}>
                        {move || {
                            user.get().map(|user| match user {
                                Ok(Some(_)) => view! {
                                    <Logout action=logout/>
                                }.into_view(),
                                _ => view! {}.into_view()
                            })
                        }}
                        </Transition>
                    </ul>
                    <FitUploadForm
                        show=show_upload
                        show_set=set_show_upload
                    />
                </div>
            </nav>
            <main>
                <div class="container">
                    <Routes>
                        <ProtectedRoute path="/" redirect_path="/login" condition=login_status view=|| view! { <Overview/> } ssr=SsrMode::PartiallyBlocked/>
                        <Route path="/login" view=move|| view! { <Login action=login/>}/>
                        <Route path="/signup" view=move|| view! { <Signup action=signup/>}/>
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

#[component]
fn Overview() -> impl IntoView {
    //overview page
    view! {
        <div class="row">
            <div class="col s12 m6 l4 p-1">
                <div class="card-panel teal">Pie Chart</div>
            </div>
            <div class="col s12 m6 l4 p-1">
                <div class="card-panel teal">Training LoadChart</div>
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
            <h1>"Log In"</h1>
            <label>
                "User ID:"
                <input type="text" placeholder="User ID" maxlength="32" name="username" class="auth-input" />
            </label>
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input" />
            </label>
            <br/>
            <label>
                <input type="checkbox" name="remember" class="auth-input" />
                "Remember me?"
            </label>
            <br/>
            <button type="submit" class="button">"Log In"</button>
            <A href="/signup">Signup</A>
        </ActionForm>
    }
}

#[component]
fn Logout(action: Action<Logout, Result<(), ServerFnError>>) -> impl IntoView {
    view! {

        <div id="loginbox">
            <ActionForm action=action>
                <button type="submit" class="button">"Log Out"</button>
            </ActionForm>
        </div>
    }
}

#[component]
fn Signup(action: Action<Signup, Result<(), ServerFnError>>) -> impl IntoView {
    view! {

        <ActionForm action=action>
            <h1>"Sign Up"</h1>
            <label>
                "User ID:"
                <input type="text" placeholder="User ID" maxlength="32" name="username" class="auth-input" />
            </label>
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input" />
            </label>
            <br/>
            <label>
                "Confirm Password:"
                <input type="password" placeholder="Password again" name="password_confirmation" class="auth-input" />
            </label>
            <br/>
            <label>
                "Remember me?"
            </label>
            <input type="checkbox" name="remember" class="auth-input" />

            <br/>
            <button type="submit" class="button">"Sign Up"</button>
        </ActionForm>
            <A href="/login">Login</A>
    }
}

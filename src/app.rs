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
    let (logged_in, set_logged_in) = create_signal(false);
    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();
    let user = create_resource(
        move || {
            (
                login.version().get(),
                logout.version().get(),
                signup.version().get(),
            )
        },
        move |_| async move {
            let user = get_user().await.unwrap_or(None);
            set_logged_in(user.is_some());
            user
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
            <nav class="teal lighten-2">
                <div class="nav-wrapper">
                    <a href="#" class="brand-logo">
                        Toedi
                    </a>
                    <ul id="nav-mobile" class="right hide-on-med-and-down">
                        <ProtectedContentWrapper
                            when=logged_in
                            fallback=move || view! { <li><A href="/login">Login</A></li><li><A href="/signup">Signup</A></li> }
                        >
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
                            <A href="/logout">Logout</A>
                        </li>
                        </ProtectedContentWrapper>
                    </ul>
                    <FitUploadForm show=show_upload show_set=set_show_upload/>
                </div>
            </nav>
            <main>
                <div class="container">
                    <Routes>
                        <Route
                            path="/"
                            view=move || {
                                view! {
                                    <ProtectedContentWrapper
                                        when=logged_in
                                        fallback=move || view! { <div>"Not logged in"</div> }
                                    >
                                        <Overview/>
                                    </ProtectedContentWrapper>
                                }
                            }
                        />
                        <Route path="/login" view=move || view! { <Login action=login/> }/>
                        <Route path="/logout" view=move || view! { <Logout action=logout/> }/>
                        <Route path="/signup" view=move || view! { <Signup action=signup/> }/>
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
                <input
                    type="text"
                    placeholder="User ID"
                    maxlength="32"
                    name="username"
                    class="auth-input"
                />
            </label>
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input"/>
            </label>
            <br/>
            <label>
                <input type="checkbox" name="remember" class="auth-input"/>
                "Remember me?"
            </label>
            <br/>
            <button type="submit" class="button">
                "Log In"
            </button>
            <A href="/signup">Signup</A>
        </ActionForm>
    }
}

#[component]
fn Logout(action: Action<Logout, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <div id="loginbox">
            <ActionForm action=action>
                <button type="submit" class="button">
                    "Log Out"
                </button>
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
                <input
                    type="text"
                    placeholder="User ID"
                    maxlength="32"
                    name="username"
                    class="auth-input"
                />
            </label>
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input"/>
            </label>
            <br/>
            <label>
                "Confirm Password:"
                <input
                    type="password"
                    placeholder="Password again"
                    name="password_confirmation"
                    class="auth-input"
                />
            </label>
            <br/>
            <label>"Remember me?"</label>
            <input type="checkbox" name="remember" class="auth-input"/>

            <br/>
            <button type="submit" class="button">
                "Sign Up"
            </button>
        </ActionForm>
        <A href="/login">Login</A>
    }
}

#[component]
pub fn ProtectedContentWrapper<F, IV>(
    fallback: F,
    children: ChildrenFn,
    when: ReadSignal<bool>,
) -> impl IntoView
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    let fallback = store_value(fallback);
    let children = store_value(children);

    view! {
        <Suspense fallback=|| ()>
            <Show when=move || when() fallback=move || fallback.with_value(|fallback| fallback())>
                {children.with_value(|children| children())}
            </Show>
        </Suspense>
    }
}

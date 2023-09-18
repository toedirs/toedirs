use crate::{
    error_template::{AppError, ErrorTemplate},
    fit_upload::FitUploadForm,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let (show_upload, set_show_upload) = create_signal(false);
    leptos::view! {
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
                        <Route path="" view=|| view! { <Overview/> }/>
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

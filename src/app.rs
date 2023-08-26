use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/toedirs.css"/>

        // sets the document title
        <Title text="Welcome to Toedi"/>

        // content for this welcome page
        <Router fallback=|cx| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { cx,
                <ErrorTemplate outside_errors/>
            }
            .into_view(cx)
        }>
            <nav class="bg-gray-800">
                <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
                    <div class="relative flex h-16 items-center justify-between">
                        <div class="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start">
                            <div class="flex flex-shrink-0 items-center text-white">
                                Toedi
                            </div>
                            <div class="hidden sm:ml-6 sm:block">
                                <div class="flex space-x-4">
                                    <A href="/" class="bg-gray-900 text-white rounded-md px-3 py-2 text-sm font-medium">Overview</A>
                                    <A href="/activities" class="bg-gray-900 text-white rounded-md px-3 py-2 text-sm font-medium">Activities</A>
                                </div>
                            </div>

                            <div class="absolute inset-y-0 right-0 flex items-center pr-2 sm:static sm:inset-auto sm:ml-6 sm:pr-0">
                                <button type="button" class="relative rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800">
                                    <span class="absolute -inset-1.5">    </span>
                                    <span class="sr-only">Upload</span>
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                                      <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
                                    </svg>
                                </button>
                            </div>                        
                        </div>
                    </div>
                </div>
            </nav>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <Overview/> }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Overview(cx: Scope) -> impl IntoView {
    //overview page
    view! {cx,
        <div class="grid grid-cols-3 gap-4">
            <div class="block bg-gray-400 h-64 rounded">
                Pie Chart
            </div>
            <div class="block bg-gray-400 h-64 rounded">
                Training LoadChart
            </div>
            <div class="block bg-gray-400 h-64 rounded">
                Fitness & Fatigue
            </div>
        </div>   
    }
}


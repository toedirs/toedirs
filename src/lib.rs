use cfg_if::cfg_if;
pub mod activity_list;
pub mod app;
pub mod auth;
pub mod config;
pub mod date_picker_dialog;
pub mod elements;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;
pub mod fit_upload;
pub mod heartrate_summary_chart;
pub mod models;
pub mod state;
pub mod training_load_chart;
pub mod workout_schedule;

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::*;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        _ = console_log::init_with_level(log::Level::Warn);
        console_error_panic_hook::set_once();

        leptos::mount_to_body(move || {
            view! { <App/> }
        });
    }
}}

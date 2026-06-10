pub mod app;
pub mod components;
pub mod tauri_bridge;

use leptos::*;

pub fn mount_app() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Failed to init log");
    mount_to_body(|| view! { <app::App /> });
}

// Leptos components return views used in macros, not directly by callers
#![allow(clippy::must_use_candidate)]

pub mod app;
pub mod pages;
pub mod types;

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod error;
#[cfg(feature = "ssr")]
pub mod phone;
#[cfg(feature = "ssr")]
pub mod sms;

#[cfg(feature = "ssr")]
pub mod assignment;

pub mod admin;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

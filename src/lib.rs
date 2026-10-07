// Leptos components return views used in macros, not directly by callers
#![allow(clippy::must_use_candidate)]
// WHY: on the hydrate target clippy 1.99 reports this lint at the `#[server]` attribute itself
// (all 30 spans, "originates in the attribute macro `server`"): the macro expands each server fn
// to an `async` trait impl whose client stub has no `.await`. The code is generated, not ours.
// `unknown_lints` covers the pinned toolchain, whose clippy predates the lint.
#![allow(unknown_lints, clippy::unused_async_trait_impl)]

pub mod app;
pub mod components;
pub mod hooks;
pub mod i18n;
pub mod pages;
pub mod types;

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod db;
pub mod error;
pub mod phone;
#[cfg(feature = "ssr")]
pub mod sms;

pub mod assignment;

#[cfg(feature = "ssr")]
pub mod date_format;

pub mod invite_codes;

pub mod admin;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

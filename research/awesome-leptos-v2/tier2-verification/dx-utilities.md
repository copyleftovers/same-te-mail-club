# DX Utilities — Source-Level Verification

## wasm-tracing

### Crate Metadata

| Field | Value |
|-------|-------|
| Latest published version | 2.1.0 |
| Total downloads | 57,159 |
| Last publish date | August 4, 2025 |
| Repository | https://github.com/dsgallups/wasm-tracing |
| Author | Daniel Gallups (@dsgallups) |
| License | MIT OR Apache-2.0 |
| Description | "Tracing subscriber for WebAssembly. Maintained fork of tracing-wasm." |

Version history shows active development from September 2024 through August 2025 (7 releases in ~11 months, with one yanked):

| Version | Date |
|---------|------|
| 2.1.0 | 2025-08-04 |
| 2.0.0 | 2025-02-27 |
| 1.0.1 | 2024-12-18 |
| 1.0.0 | 2024-12-18 (yanked) |
| 0.2.1 | 2024-09-04 |
| 0.2.0 | 2024-09-04 |
| 0.1.0 | 2024-09-04 |

**Note:** The `main` branch Cargo.toml shows version `3.0.0-alpha.0` — an unreleased alpha is already in progress as of the fetch date.

### Dependencies

From the published `main` branch Cargo.toml (which corresponds to the 3.0.0-alpha, but the `^0.1` / `^0.3` bounds are semver-compatible with what was published in 2.1.0):

```toml
[dependencies]
rayon = { version = "1.10", optional = true }
tracing = { version = "0.1", features = ["attributes"], default-features = false }
tracing-subscriber = { version = "0.3", features = ["registry"], default-features = false }
wasm-bindgen = "0.2"

[features]
mark-with-rayon-thread-index = ["dep:rayon"]
```

`tracing-log` is an optional dependency (not enabled by default).

### Compatibility with Project

| Dependency | Project version | wasm-tracing requirement | Compatible? |
|-----------|----------------|--------------------------|-------------|
| `tracing` | 0.1.44 | `^0.1` | Yes — exact same minor series |
| `tracing-subscriber` | 0.3.22 | `^0.3` | Yes — exact same minor series |
| `wasm-bindgen` | transitive via leptos | `^0.2` | Yes — leptos already uses 0.2.x |

No version conflicts. The `^0.1` and `^0.3` semver requirements subsume the project's pinned patch versions.

### Source Analysis

`wasm-tracing` implements `tracing_subscriber::layer::Layer` via a `WasmLayer` struct. Size: 15,802 bytes (the entire published crate).

**What it does:**

- `WasmLayer` implements the Layer trait with five hooks: `enabled`, `on_new_span`, `on_record`, `on_event`, `on_enter`/`on_exit`
- `on_event` formats the tracing event into a string and calls into `window.console.log/debug/warn/error` via wasm-bindgen FFI, with optional color styling via CSS strings
- `on_enter`/`on_exit` call `window.performance.mark()` and `window.performance.measure()` to create browser performance timeline entries
- `WasmLayerConfig` exposes builder-style configuration: `max_level`, `color` output, `show_origin` (file:line), `show_fields`, `report_logs_in_timings`, `origin_base_url`
- Default `max_level` is `TRACE` (all events pass through)

**Public API surface:**

```rust
pub fn set_as_global_default()
pub fn set_as_global_default_with_config(config: WasmLayerConfig)
pub fn try_set_as_global_default() -> Result<(), SetGlobalDefaultError>
pub struct WasmLayer
pub struct WasmLayerConfig  // builder pattern
pub enum ConsoleConfig
pub mod prelude  // re-exports WasmLayer, WasmLayerConfig
```

### Integration Code

For a Leptos 0.8 `hydrate()` function in `src/app.rs` or wherever the hydrate entry point lives:

```rust
// In Cargo.toml (client-side only — under [target.'cfg(target_arch = "wasm32")'.dependencies])
// or as a regular dep since wasm-tracing is WASM-only anyway:
wasm-tracing = "2.1.0"

// In your hydrate() function or #[wasm_bindgen(start)] entry point:
#[cfg(target_arch = "wasm32")]
pub fn hydrate() {
    // Initialize panic hook for readable WASM panics
    console_error_panic_hook::set_once();  // needs console_error_panic_hook dep

    // Minimal setup — uses defaults (max_level=TRACE, color=true, timings=true)
    wasm_tracing::set_as_global_default();

    // — OR with custom config —
    use wasm_tracing::WasmLayerConfig;
    wasm_tracing::set_as_global_default_with_config(
        WasmLayerConfig::new()
            .with_max_level(tracing::Level::DEBUG)
            .with_colorless_logs()    // removes CSS color styling
            .remove_timings()         // disables performance.mark/measure
            .remove_origin(),         // hides file:line in output
    );

    leptos::mount::hydrate_body(App);
}
```

`set_as_global_default()` calls `tracing::subscriber::set_global_default()` internally and panics on double-init. If there's already a global subscriber (possible if you compose layers), use `try_set_as_global_default()` or compose `WasmLayer` into your subscriber stack manually:

```rust
use tracing_subscriber::prelude::*;
use wasm_tracing::WasmLayer;

tracing_subscriber::registry()
    .with(WasmLayer::new_with_config(WasmLayerConfig::new()))
    .init();
```

### vs tracing-subscriber-wasm

These are **not the same codebase**. `wasm-tracing` self-describes as "maintained fork of tracing-wasm" — it forks `tracing-wasm 0.2.1` (last published 2021-11-07, 7.7M downloads), not `tracing-subscriber-wasm`.

| Dimension | `tracing-subscriber-wasm 0.1.0` | `wasm-tracing 2.1.0` |
|-----------|--------------------------------|----------------------|
| Last publish | 2023-01-31 | 2025-08-04 |
| Total downloads | 176,065 | 57,159 |
| Forked from | Independent (not a fork) | `tracing-wasm 0.2.1` |
| Implementation approach | `MakeWriter` — plugs into `tracing_subscriber::fmt` layer via `.with_writer()` | Custom `Layer` — implements `tracing_subscriber::layer::Layer` directly, bypassing fmt |
| Console routing | Uses `gloo::console` (wraps wasm-bindgen under the hood) | Direct `wasm-bindgen` FFI to `window.console` |
| Performance timings | No | Yes — `window.performance.mark/measure` |
| Color output | No | Yes — CSS-styled console output |
| Configuration | Level remapping per-channel | `WasmLayerConfig` builder with max_level, origin, fields, timings, color |
| Dependencies | `gloo 0.8`, `tracing 0.1`, `tracing-subscriber 0.3` | `tracing 0.1`, `tracing-subscriber 0.3`, `wasm-bindgen 0.2` |
| SSR safety | Must be guarded — will panic on server if gloo tries to access browser APIs | Must be guarded — wasm-bindgen FFI will fail on server |
| Active maintenance | No (single release, 2+ years stale) | Yes (7 releases, latest 2025-08-04) |

**Key difference in architecture:** `tracing-subscriber-wasm` is a `MakeWriter` — you compose it as `SubscriberBuilder::with_writer(MakeConsoleWriter::new())`. `wasm-tracing` is a full `Layer` — you add it to a `Registry` or use `set_as_global_default()`. These are different integration points in the tracing stack. You cannot substitute one for the other without changing your subscriber setup code.

The prior agent's claim that `wasm-tracing` is "an active maintained fork of `tracing-subscriber-wasm`" is **incorrect**. It forks `tracing-wasm`, not `tracing-subscriber-wasm`. Both solve the same problem (WASM console logging) via different internal designs.

---

## leptos-use Hooks Relevant to Project

### Available Hooks (full list)

leptos-use 0.18.3 exports 89 functions total. Complete `use_*` inventory (from docs.rs):

`use_active_element`, `use_breakpoints`, `use_broadcast_channel`, `use_calendar`, `use_clipboard`, `use_color_mode`, `use_cookie`, `use_css_var`, `use_cycle_list`, `use_debounce_fn`, `use_device_orientation`, `use_device_pixel_ratio`, `use_display_media`, `use_document`, `use_document_visibility`, `use_draggable`, `use_drop_zone`, `use_element_bounding`, `use_element_hover`, `use_element_size`, `use_element_visibility`, `use_event_listener`, `use_event_source`, `use_favicon`, `use_geolocation`, `use_idle`, `use_infinite_scroll`, `use_intersection_observer`, `use_interval`, `use_interval_fn`, `use_intl_number_format`, `use_locale`, `use_locales`, `use_media_query`, `use_mouse`, `use_mouse_in_element`, `use_mutation_observer`, `use_permission`, `use_preferred_contrast`, `use_preferred_dark`, `use_prefers_reduced_motion`, `use_raf_fn`, `use_resize_observer`, `use_screen_orientation`, `use_scroll`, `use_service_worker`, `use_sorted`, `use_supported`, `use_textarea_autosize`, `use_throttle_fn`, `use_timeout_fn`, `use_timestamp`, `use_to_string`, `use_toggle`, `use_user_media`, `use_web_lock`, `use_web_notification`, `use_websocket`, `use_window`, `use_window_focus`, `use_window_scroll`, `use_window_size`

Plus reactive utilities: `signal_debounced`, `signal_throttled`, `sync_signal`, `watch_debounced`, `watch_pausable`, `watch_throttled`, `watch_with_options`, `whenever` (and `_with_options` / `_local` / `_with_arg` variants of many of the above).

### Hooks That Address Project Boilerplate

#### Hydration Detection — NO HOOK EXISTS

- **Project pain point:** 9 instances of:
  ```rust
  let (hydrated, set_hydrated) = signal(false);
  Effect::new(move |_| set_hydrated.set(true));
  ```
- **Gap confirmed:** `leptos-use 0.18.3` has **no `use_mounted` or `use_hydrated` hook**. The closest candidates are `use_window` and `use_document` (both return `None` on server, `Some(...)` on client), and `use_supported`. None of these produce a `Signal<bool>` tracking the transition from unhydrated to hydrated in the same way.
- `use_supported` takes a callback returning `bool` — it could theoretically be used as `use_supported(|| true)` to get a signal that is `false` on server and `true` on client, but this is a semantic misuse and produces different SSR semantics than `Effect::new`.
- The project boilerplate is 4 lines per instance and is already the idiomatic Leptos 0.8 pattern. No replacement available in leptos-use.

#### use_media_query

- **Project pain point it addresses:** Any future responsive behavior controlled by CSS media queries, or programmatic response to `(prefers-color-scheme: dark)`.
- **API:**
  ```rust
  pub fn use_media_query(query: impl Into<Signal<String>>) -> Signal<bool>
  ```
- **SSR behavior:** Returns a `Signal` that is always `false` on server (documented explicitly). Client-side, it attaches to `window.matchMedia`.
- **Already a transitive dep** — leptos-use 0.18.3 is present via leptos_i18n, so this is zero additional dependency cost.
- **Code example:**
  ```rust
  // Current project approach for dark mode (CSS-only, no Rust signal):
  // @media (prefers-color-scheme: dark) { ... } in tailwind.css

  // With use_media_query (only useful if Rust logic must branch on dark mode):
  let is_dark = use_media_query("(prefers-color-scheme: dark)");
  // is_dark.get() is false during SSR, true/false after hydration
  ```
- **Caveat:** The project's current dark mode is CSS-only via `@media`. `use_media_query` adds a Rust signal layer only if JS-driven class toggling or conditional rendering is needed. The current design does not require this.

#### use_preferred_dark

- **Project pain point it addresses:** Same as `use_media_query("(prefers-color-scheme: dark)")` but with SSR header reading.
- **API:**
  ```rust
  pub fn use_preferred_dark() -> Signal<bool>
  ```
- **SSR behavior:** Reads `Sec-CH-Prefers-Color-Scheme` HTTP header. If absent, defaults to `false` (Light). Requires the `"axum"` feature flag on leptos-use to access the request header in Axum context.
- **Feature flag check needed:** The project has `leptos-use` as a transitive dep via leptos_i18n. Whether the `"axum"` feature is enabled in that transitive pull is not guaranteed — would need to be verified in the lock file or by adding leptos-use as a direct dep with `features = ["axum"]`.
- **Code example:**
  ```rust
  let is_dark = use_preferred_dark();
  // SSR: reads Sec-CH-Prefers-Color-Scheme header (needs axum feature)
  // Client: reactive to window.matchMedia('(prefers-color-scheme: dark)')
  ```

#### use_color_mode

- **Project pain point it addresses:** Full dark/light mode management with storage persistence and DOM class toggling.
- **API:**
  ```rust
  pub fn use_color_mode() -> UseColorModeReturn
  // Returns: { mode, set_mode, store, set_store, system, state }
  ```
- **SSR behavior:** Reads `Sec-CH-Prefers-Color-Scheme` header (same as `use_preferred_dark`). Requires `"axum"` feature. Client-side uses `use_storage` to persist to localStorage by default.
- **What it does:** Manages a user-selectable color mode preference with localStorage persistence. Applies a CSS class (or `data-*` attribute) to a target element. The project's design system uses `@media (prefers-color-scheme: dark)` only — no manual toggle, no persistence. `use_color_mode` is overbuilt for the current requirement.

#### use_debounce_fn

- **Project pain point it addresses:** Future search/filter features where input events should not trigger on every keystroke.
- **API:**
  ```rust
  pub fn use_debounce_fn<F, R>(func: F, ms: impl Into<Signal<f64>>) -> impl Fn() -> Arc<Mutex<Option<R>>>
  ```
- **SSR behavior:** Relies on `setTimeout`. Server-side calls are silently ignored per docs.
- **No current project pain point** — the project has no search or typeahead features. This addresses a future scenario.
- **Code example (hypothetical):**
  ```rust
  let mut debounced_search = use_debounce_fn(
      move || { /* trigger server fn */ },
      300.0,
  );
  view! {
      <input on:input=move |_| { debounced_search(); } />
  }
  ```

#### use_throttle_fn

- **Project pain point it addresses:** Same future scenarios as `use_debounce_fn` — rate-limiting event handlers.
- **API:**
  ```rust
  pub fn use_throttle_fn<F, R>(func: F, ms: impl Into<Signal<f64>>) -> impl Fn() -> Arc<Mutex<Option<R>>>
  ```
- **SSR behavior:** Not explicitly documented, but uses `setTimeout`-based mechanism — same as debounce; no-op on server.
- **No current project pain point** — no scroll handlers, no resize handlers, no high-frequency events in current implementation.

#### use_window / use_document

- **Project pain point it addresses:** SSR-safe access to `window` and `document` objects if needed from Rust.
- **API:**
  ```rust
  pub fn use_window() -> UseWindow   // UseWindow wraps Option<Window>
  pub fn use_document() -> UseDocument  // UseDocument wraps Option<Document>
  ```
- **SSR behavior:** Returns `None`-wrapping type on server, `Some`-wrapping on client. Prevents panics from `web_sys::window().unwrap()`.
- **Current project usage:** Zero instances of `web_sys::window()` or `web_sys::document()` direct calls found in the project — the hydration gate via `Effect::new` already avoids needing direct window access. No current pain point.

#### use_prefers_reduced_motion

- **Project pain point it addresses:** The design system requires `prefers-reduced-motion: reduce` to hide grain overlay and disable transitions — currently handled in CSS only.
- **API:**
  ```rust
  pub fn use_prefers_reduced_motion() -> Signal<bool>
  // (variant: use_prefers_reduced_motion_with_options)
  ```
- **SSR behavior:** Same SSR caveat as `use_media_query` — returns `false` on server. Only relevant if Rust-level branching on motion preference is needed.
- **Current implementation:** Pure CSS via `@media (prefers-reduced-motion: reduce)`. No Rust signal needed unless JS-driven behavior branches on this value. No current pain point.

### Hooks NOT Available (gaps)

| Need | Status |
|------|--------|
| `use_hydrated` / `use_mounted` — `Signal<bool>` that is `false` during SSR and becomes `true` after WASM hydration | **Not present.** The 4-line `signal(false) + Effect::new` pattern is the idiomatic Leptos 0.8 approach and has no leptos-use equivalent. |
| Hydration gate as a composable primitive | **Not present.** No function returns a `ReadSignal<bool>` tied to the client-hydration lifecycle. |
| Form field error display with ARIA wiring | **Not in leptos-use scope.** leptos-use is browser API bindings, not form component library. |
| Context extraction helpers (pool, request parts) | **Not in leptos-use scope.** Server-side plumbing. |

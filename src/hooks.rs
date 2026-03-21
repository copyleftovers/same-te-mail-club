use leptos::prelude::*;

/// Returns a signal that is `false` during SSR and the hydration gap,
/// then `true` once WASM has hydrated the page.
///
/// Use this to gate interactive elements (e.g. `disabled=move || !hydrated.get()`)
/// so Playwright's auto-wait synchronizes with hydration.
pub fn use_hydrated() -> ReadSignal<bool> {
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });
    hydrated
}

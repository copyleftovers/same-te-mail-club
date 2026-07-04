use leptos::prelude::*;
use std::time::Duration;

/// Toast state provided via context.
///
/// Holds a signal pair for the current toast message.
/// The write signal is extracted by components via `use_toast()`.
#[derive(Clone, Copy)]
pub struct ToastState {
    pub read: ReadSignal<Option<String>>,
    pub write: WriteSignal<Option<String>>,
}

/// Provides toast context and returns the state.
///
/// Call this once in the app shell before rendering any components.
pub fn provide_toast_context() -> ToastState {
    let (read, write) = signal(None::<String>);
    let state = ToastState { read, write };
    provide_context(state);
    state
}

/// Extract the toast write signal from context.
///
/// Pages call this to dispatch toast messages.
pub fn use_toast() -> WriteSignal<Option<String>> {
    expect_context::<ToastState>().write
}

/// Toast notification component.
///
/// Renders a sticky flow-banner at the top of `<main>` (not a fixed overlay).
/// Visible only while the signal holds a message.
/// Auto-dismisses after 4 seconds with a slide-out exit animation.
#[component]
pub fn Toast() -> impl IntoView {
    let state = expect_context::<ToastState>();
    let message = state.read;

    // `leaving` drives [data-state="leaving"] so the slide-out keyframe plays
    // before the element is actually removed from the DOM.
    let leaving = RwSignal::new(false);

    // Stored so we can cancel a pending dismiss when the toast re-fires.
    let dismiss_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let exit_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);

    // Auto-dismiss: client-side only (Effect never runs on SSR).
    // On each new message: cancel any in-flight timers, reset `leaving`,
    // then schedule the exit sequence (mark leaving → clear message).
    Effect::new(move |_| {
        if message.get().is_some() {
            // Cancel leftover timers from a previous message.
            if let Some(h) = dismiss_handle.get_value() {
                h.clear();
            }
            if let Some(h) = exit_handle.get_value() {
                h.clear();
            }
            leaving.set(false);

            // After ~3.85s begin the exit animation (~150ms), then clear at ~4s.
            let exit_h =
                set_timeout_with_handle(move || leaving.set(true), Duration::from_millis(3850))
                    .expect("set_timeout exit");
            exit_handle.set_value(Some(exit_h));

            let dismiss_h = set_timeout_with_handle(
                move || {
                    state.write.set(None);
                    leaving.set(false);
                },
                Duration::from_millis(4000),
            )
            .expect("set_timeout dismiss");
            dismiss_handle.set_value(Some(dismiss_h));
        }
    });

    // Clear pending timers when the component unmounts.
    on_cleanup(move || {
        if let Some(h) = dismiss_handle.try_get_value().flatten() {
            h.clear();
        }
        if let Some(h) = exit_handle.try_get_value().flatten() {
            h.clear();
        }
    });

    view! {
        <div
            role="status"
            aria-live="polite"
            aria-atomic="true"
            class="toast-container"
        >
            {move || message.get().map(|msg| view! {
                <div
                    class="toast"
                    data-testid="toast"
                    data-state=move || if leaving.get() { "leaving" } else { "visible" }
                >
                    <p class="toast-message">{msg}</p>
                </div>
            })}
        </div>
    }
}

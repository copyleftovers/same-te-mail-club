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
/// Renders a fixed-position toast at the bottom of the viewport.
/// Only renders when the signal contains a message.
/// Auto-dismisses after 5 seconds.
#[component]
pub fn Toast() -> impl IntoView {
    let state = expect_context::<ToastState>();
    let message = state.read;

    // Auto-dismiss timer (client-side only via Effect)
    Effect::new(move |_| {
        if message.get().is_some() {
            set_timeout(
                move || {
                    state.write.set(None);
                },
                Duration::from_secs(5),
            );
        }
    });

    view! {
        <div
            role="status"
            aria-live="polite"
            aria-atomic="true"
            class="fixed bottom-4 left-4 right-4 z-40 sm:left-auto sm:max-w-96"
        >
            {move || message.get().map(|msg| view! {
                <div class="toast" data-testid="toast">
                    <p class="toast-message">{msg}</p>
                </div>
            })}
        </div>
    }
}

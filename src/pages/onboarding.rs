use crate::error::{FIELD_DISCRIMINANT_SEPARATOR, strip_server_error_prefix};
use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, use_i18n};
use leptos::prelude::*;
use leptos_use::use_preferred_dark;

// ── Server function ───────────────────────────────────────────────────────────

/// Save the user's Nova Poshta delivery address and mark them as onboarded.
///
/// Takes separate city and branch number inputs.
///
/// # Errors
///
/// Returns `Err` if the session is invalid, input is empty, or DB fails.
/// Validation errors are encoded as `"field_key\u{1f}localized_message"` so
/// the client can route to the correct field without inspecting locale-specific prose.
#[server]
pub async fn complete_onboarding(city: String, np_number: String) -> Result<(), ServerFnError> {
    use crate::auth;
    use crate::i18n::i18n::{Locale, td_string};

    let (pool, user) = auth::require_auth().await?;

    let city = city.trim().to_owned();
    if city.is_empty() {
        return Err(ServerFnError::new(format!(
            "city{SEP}{msg}",
            SEP = FIELD_DISCRIMINANT_SEPARATOR,
            msg = td_string!(Locale::uk, onboarding_error_city_required),
        )));
    }
    let number: i32 = np_number.trim().parse().map_err(|_| {
        ServerFnError::new(format!(
            "np_number{SEP}{msg}",
            SEP = FIELD_DISCRIMINANT_SEPARATOR,
            msg = td_string!(Locale::uk, onboarding_error_np_number_invalid),
        ))
    })?;
    if number < 1 {
        return Err(ServerFnError::new(format!(
            "np_number{SEP}{msg}",
            SEP = FIELD_DISCRIMINANT_SEPARATOR,
            msg = td_string!(Locale::uk, onboarding_error_np_number_invalid),
        )));
    }

    // Upsert delivery address
    sqlx::query!(
        r#"
        INSERT INTO delivery_addresses (user_id, nova_poshta_city, nova_poshta_number)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) DO UPDATE
            SET nova_poshta_city = EXCLUDED.nova_poshta_city,
                nova_poshta_number = EXCLUDED.nova_poshta_number,
                updated_at = now()
        "#,
        user.id,
        city,
        number,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Mark user as onboarded
    sqlx::query!("UPDATE users SET onboarded = true WHERE id = $1", user.id,)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    leptos_axum::redirect("/");
    Ok(())
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Which field the server rejected.
#[derive(Clone, PartialEq)]
enum RejectedField {
    City,
    NpNumber,
}

/// Split a stripped server error into `(field, display_message)`.
///
/// The server encodes validation errors as `"field_key\u{1f}message"`.
/// Returns `(Some(field), message)` when the separator is present and the key
/// is recognised; `(None, full_string)` for infra/session errors with no
/// separator — those mark no field and display the raw message.
fn parse_field_error(stripped: &str) -> (Option<RejectedField>, &str) {
    if let Some((key, msg)) = stripped.split_once(FIELD_DISCRIMINANT_SEPARATOR) {
        let field = match key {
            "city" => Some(RejectedField::City),
            "np_number" => Some(RejectedField::NpNumber),
            _ => None,
        };
        (field, msg)
    } else {
        (None, stripped)
    }
}

/// Onboarding form: collect Nova Poshta delivery address.
/// Shown only on first login; navigates to `/` on success via a full page
/// reload so SSR re-runs `get_current_user()` with `onboarded=true`.
// The `view!` macro's HTML attribute verbosity inflates line count beyond what
// reflects logic complexity; extracting sub-components here would be YAGNI.
#[allow(clippy::too_many_lines)]
#[component]
pub fn OnboardingPage() -> impl IntoView {
    let i18n = use_i18n();
    // (message, rejected field) — both None until the first error
    let (city_error, set_city_error) = signal(Option::<String>::None);
    let (np_error, set_np_error) = signal(Option::<String>::None);

    let onboard_action = ServerAction::<CompleteOnboarding>::new();
    let onboard_pending = onboard_action.pending();

    let hydrated = use_hydrated();

    // Dark-mode-aware hero logo — mirrors login.rs's swap (same asset pair, same mechanism).
    let is_dark = use_preferred_dark();

    Effect::new(move |_| {
        if let Some(result) = onboard_action.value().get() {
            match result {
                Ok(()) => {
                    let _ = leptos::prelude::window().location().set_href("/");
                }
                Err(e) => {
                    let stripped = strip_server_error_prefix(&e);
                    let (field, display_msg) = parse_field_error(&stripped);
                    let msg = display_msg.to_owned();
                    match field {
                        Some(RejectedField::City) => {
                            set_city_error.set(Some(msg));
                            set_np_error.set(None);
                        }
                        Some(RejectedField::NpNumber) => {
                            set_np_error.set(Some(msg));
                            set_city_error.set(None);
                        }
                        None => {
                            set_city_error.set(None);
                            set_np_error.set(None);
                        }
                    }
                }
            }
        }
    });

    view! {
        // Onboarding is the mandatory single step reached right after registration/login —
        // it shares the auth flow's shell (.page-frame > .auth-card) so its vertical
        // placement and typography match login rather than the 65ch .prose-page column.
        <div class="page-frame">
            <div class="auth-card">
                // Icon-only mark — mirrors login.rs's hero swap (same asset pair,
                // same rationale: the compound logo.svg embeds an illegible
                // wordmark; the <h1> below already carries the readable label).
                <img
                    src=move || {
                        if is_dark.get() {
                            "/same_te_mark_white.svg"
                        } else {
                            "/same_te_mark_orange.svg"
                        }
                    }
                    alt="Саме Те · Поштовий клуб"
                />
                // Wrapped in a plain div (mirroring login.rs's per-step wrapper)
                // so the heading resolves against .auth-card > div { width:100% }
                // instead of shrink-wrapping as a bare flex child of .auth-card —
                // the same centered treatment login's step headings get (RV-10).
                <div>
                    <h1>{t!(i18n, onboarding_page_title)}</h1>
                    <p>{t!(i18n, onboarding_description)}</p>

                    <leptos::form::ActionForm action=onboard_action>
                        <div class="flex flex-col gap-(--density-space-md)">
                            <div class="field w-full">
                                <label class="field-label" for="np-city">
                                    {t!(i18n, onboarding_city_label)}
                                </label>
                                <input
                                    class="field-input"
                                    type="text"
                                    id="np-city"
                                    name="city"
                                    placeholder="Київ"
                                    required
                                    data-testid="np-city-input"
                                    aria-invalid=move || city_error.get().map(|_| "true")
                                    aria-describedby="np-city-error"
                                />
                                <p
                                    id="np-city-error"
                                    class="field-error"
                                    aria-live="assertive"
                                    data-testid="np-city-error"
                                >
                                    {move || city_error.get()}
                                </p>
                            </div>
                            <div class="field w-full">
                                <label class="field-label" for="np-number">
                                    {t!(i18n, onboarding_np_number_label)}
                                </label>
                                <input
                                    class="field-input"
                                    id="np-number"
                                    type="text"
                                    inputmode="numeric"
                                    name="np_number"
                                    placeholder="123"
                                    required
                                    data-testid="np-number-input"
                                    aria-invalid=move || np_error.get().map(|_| "true")
                                    aria-describedby="np-number-error"
                                />
                                <p
                                    id="np-number-error"
                                    class="field-error"
                                    aria-live="assertive"
                                    data-testid="np-number-error"
                                >
                                    {move || np_error.get()}
                                </p>
                            </div>
                        </div>
                        <button
                            class="btn w-full mt-(--density-space-lg)"
                            type="submit"
                            data-testid="save-onboarding-button"
                            disabled=move || onboard_pending.get() || !hydrated.get()
                        >
                            {move || if onboard_pending.get() {
                                "Зберігаю...".into_any()
                            } else {
                                t!(i18n, onboarding_save_button).into_any()
                            }}
                        </button>
                    </leptos::form::ActionForm>
                </div>
            </div>
        </div>
    }
}

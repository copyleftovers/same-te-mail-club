use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, t_string, use_i18n};
use crate::pages::login::strip_server_error_prefix;
use leptos::prelude::*;

// ── Server function ───────────────────────────────────────────────────────────

/// Save the user's Nova Poshta delivery address and mark them as onboarded.
///
/// Takes separate city and branch number inputs.
///
/// # Errors
///
/// Returns `Err` if the session is invalid, input is empty, or DB fails.
#[server]
pub async fn complete_onboarding(city: String, np_number: String) -> Result<(), ServerFnError> {
    use crate::auth;

    let (pool, user) = auth::require_auth().await?;

    let city = city.trim().to_owned();
    if city.is_empty() {
        return Err(ServerFnError::new("city is required"));
    }
    let number: i32 = np_number
        .trim()
        .parse()
        .map_err(|_| ServerFnError::new("invalid branch number"))?;
    if number < 1 {
        return Err(ServerFnError::new("branch number must be positive"));
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

/// Which field the server rejected. Derived from the error message text so the
/// rejected field can get its own `aria-invalid` and error container.
#[derive(Clone, PartialEq)]
enum RejectedField {
    City,
    NpNumber,
}

/// Parse the stripped server error to determine which field was rejected.
///
/// Returns `None` for infra/session errors that name no field — those clear both
/// `aria-invalid` signals rather than falsely flagging a field. Validation errors:
/// "city is required" → `Some(City)`; "branch …" → `Some(NpNumber)`.
fn rejected_field_from_error(stripped: &str) -> Option<RejectedField> {
    if stripped.contains("city") {
        Some(RejectedField::City)
    } else if stripped.contains("branch") {
        Some(RejectedField::NpNumber)
    } else {
        None
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

    Effect::new(move |_| {
        if let Some(result) = onboard_action.value().get() {
            match result {
                Ok(()) => {
                    let _ = leptos::prelude::window().location().set_href("/");
                }
                Err(e) => {
                    let stripped = strip_server_error_prefix(&e);
                    let msg = format!("{}{}", t_string!(i18n, onboarding_error_prefix), stripped);
                    match rejected_field_from_error(&stripped) {
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
        // pt-[10svh]: viewport-relative top padding matches login layout for visual continuity
        <div class="prose-page flex flex-col pt-[10svh]">
            <div class="flex flex-col items-center text-center">
                <h1>{t!(i18n, onboarding_page_title)}</h1>
                <p>{t!(i18n, onboarding_description)}</p>
            </div>

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
                        <div
                            id="np-city-error"
                            aria-live="assertive"
                            data-testid="action-error"
                        >
                            {move || city_error.get().map(|msg| view! {
                                <p class="field-error" role="alert">{msg}</p>
                            })}
                        </div>
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
                            aria-describedby="np-np-number-error"
                        />
                        <div
                            id="np-np-number-error"
                            aria-live="assertive"
                            data-testid="np-number-error"
                        >
                            {move || np_error.get().map(|msg| view! {
                                <p class="field-error" role="alert">{msg}</p>
                            })}
                        </div>
                    </div>
                </div>
                <button
                    class="btn w-full mt-(--density-space-md)"
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
    }
}

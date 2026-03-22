use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, t_string, use_i18n};
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

/// Onboarding form: collect Nova Poshta delivery address.
/// Shown only on first login; redirects to `/` on success via server redirect.
///
/// Uses `ActionForm` which reads form values from DOM directly.
/// The server issues a redirect on success, which `ActionForm`'s redirect
/// hook handles via `window.location.set_href("/")`.
#[component]
pub fn OnboardingPage() -> impl IntoView {
    let i18n = use_i18n();
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let onboard_action = ServerAction::<CompleteOnboarding>::new();
    let onboard_pending = onboard_action.pending();

    let hydrated = use_hydrated();

    Effect::new(move |_| {
        if let Some(Err(e)) = onboard_action.value().get() {
            set_error_msg.set(Some(format!(
                "{}{e}",
                t_string!(i18n, onboarding_error_prefix)
            )));
        }
    });

    view! {
        <div class="prose-page">
            <h1>{t!(i18n, onboarding_page_title)}</h1>
            <p>{t!(i18n, onboarding_description)}</p>

            <leptos::form::ActionForm action=onboard_action>
                <div class="flex flex-col gap-(--density-space-md) sm:flex-row sm:gap-(--density-space-sm)">
                    <div class="field sm:w-1/2">
                        <label class="field-label" for="np-city">
                            {t!(i18n, onboarding_city_label)}
                        </label>
                        <input
                            class="field-input"
                            type="text"
                            id="np-city"
                            name="city"
                            value="Київ"
                            required
                            data-testid="np-city-input"
                            aria-invalid=move || error_msg.get().is_some()
                            aria-describedby="np-branch-error"
                        />
                    </div>
                    <div class="field sm:w-1/2">
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
                            aria-invalid=move || error_msg.get().is_some()
                            aria-describedby="np-branch-error"
                        />
                    </div>
                </div>
                <div
                    id="np-branch-error"
                    role="alert"
                    aria-live="assertive"
                    data-testid="action-error"
                >
                    {move || error_msg.get().map(|msg| view! { <span>{msg}</span> })}
                </div>
                <button
                    class="btn"
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

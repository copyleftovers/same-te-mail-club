use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

// ── Server function ───────────────────────────────────────────────────────────

/// Save the user's Nova Poshta delivery address and mark them as onboarded.
///
/// Takes a single branch string like "Відділення №1, Київ" and parses city/number from it.
///
/// # Errors
///
/// Returns `Err` if the session is invalid, input is empty, or DB fails.
#[server]
pub async fn complete_onboarding(branch: String) -> Result<(), ServerFnError> {
    use crate::auth;
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let trimmed = branch.trim().to_owned();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("Nova Poshta branch is required"));
    }

    // Parse number from branch text (e.g. "Відділення №1, Київ" → 1)
    let number: i32 = trimmed
        .chars()
        .skip_while(|c: &char| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(1);

    // Extract city: everything after the first ", " separator
    // "Відділення №5, Київ" → "Київ"
    // "№5, Київ" → "Київ"
    // "Відділення №5" → "" (no city — store empty string)
    let city = trimmed
        .split_once(", ")
        .map_or("", |x| x.1)
        .trim()
        .to_owned();

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

    // Hydration gate
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

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
                <div class="field">
                    <label class="field-label" for="np-branch">{t!(i18n, onboarding_branch_label)}</label>
                    <input
                        class="field-input"
                        id="np-branch"
                        type="text"
                        name="branch"
                        placeholder=move || t_string!(i18n, onboarding_branch_placeholder)
                        data-testid="branch-input"
                        aria-invalid=move || error_msg.get().is_some()
                        aria-describedby="np-branch-error"
                    />
                    <div id="np-branch-error" role="alert" aria-live="assertive" data-testid="action-error">
                        {move || error_msg.get().map(|msg| view! { <span>{msg}</span> })}
                    </div>
                </div>
                <button class="btn" type="submit" data-testid="save-onboarding-button" disabled=move || !hydrated.get()>
                    {t!(i18n, onboarding_save_button)}
                </button>
            </leptos::form::ActionForm>
        </div>
    }
}

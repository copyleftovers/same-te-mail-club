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
        trimmed,
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
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let onboard_action = ServerAction::<CompleteOnboarding>::new();

    // Hydration gate
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

    Effect::new(move |_| {
        if let Some(Err(e)) = onboard_action.value().get() {
            set_error_msg.set(Some(format!("Помилка: {e}")));
        }
    });

    view! {
        <div class="onboarding-page">
            <h1>"Налаштування акаунту"</h1>
            <p>"Вкажіть ваше відділення Nova Poshta для отримання посилок."</p>

            <leptos::form::ActionForm action=onboard_action>
                <label for="np-branch">"Nova Poshta відділення (branch)"</label>
                <input
                    id="np-branch"
                    type="text"
                    name="branch"
                    placeholder="Відділення №1, Київ"
                />
                <button type="submit" disabled=move || !hydrated.get()>
                    "Save and continue"
                </button>
            </leptos::form::ActionForm>

            {move || error_msg.get().map(|msg| view! {
                <p class="error">{msg}</p>
            })}
        </div>
    }
}

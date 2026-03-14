use leptos::prelude::*;

// ── Server function ───────────────────────────────────────────────────────────

/// Save the user's Nova Poshta delivery address and mark them as onboarded.
///
/// # Errors
///
/// Returns `Err` if the session is invalid, input is invalid, or DB fails.
#[server]
pub async fn complete_onboarding(
    nova_poshta_city: String,
    nova_poshta_number: i32,
) -> Result<(), ServerFnError> {
    use crate::auth;
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let city = nova_poshta_city.trim().to_owned();
    if city.is_empty() {
        return Err(ServerFnError::new("Nova Poshta city is required"));
    }
    if nova_poshta_number <= 0 {
        return Err(ServerFnError::new("Nova Poshta number must be positive"));
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
        nova_poshta_number,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Mark user as onboarded
    sqlx::query!("UPDATE users SET onboarded = true WHERE id = $1", user.id,)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Onboarding form: collect Nova Poshta delivery address.
/// Shown only on first login; redirects to `/` on success.
#[component]
pub fn OnboardingPage() -> impl IntoView {
    let (city, set_city) = signal(String::new());
    let (branch_number, set_branch_number) = signal(1i32);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let onboard_action = ServerAction::<CompleteOnboarding>::new();

    Effect::new(move |_| {
        if let Some(result) = onboard_action.value().get() {
            match result {
                Ok(()) => {
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/", leptos_router::NavigateOptions::default());
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Помилка: {e}")));
                }
            }
        }
    });

    view! {
        <div class="onboarding-page">
            <h1>"Налаштування акаунту"</h1>
            <p>"Вкажіть ваше відділення Nova Poshta для отримання посилок."</p>

            <form on:submit=move |ev| {
                ev.prevent_default();
                onboard_action.dispatch(CompleteOnboarding {
                    nova_poshta_city: city.get(),
                    nova_poshta_number: branch_number.get(),
                });
            }>
                <div>
                    <label for="np-city">"Nova Poshta відділення (місто)"</label>
                    <input
                        id="np-city"
                        type="text"
                        name="nova_poshta_city"
                        placeholder="Київ"
                        on:input=move |ev| set_city.set(event_target_value(&ev))
                        prop:value=city
                    />
                </div>
                <div>
                    <label for="np-number">"Номер відділення"</label>
                    <input
                        id="np-number"
                        type="number"
                        name="nova_poshta_number"
                        min="1"
                        prop:value=branch_number
                        on:input=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<i32>() {
                                set_branch_number.set(n);
                            }
                        }
                    />
                </div>

                <button type="submit">"Save and continue"</button>
            </form>

            {move || error_msg.get().map(|msg| view! {
                <p class="error">{msg}</p>
            })}
        </div>
    }
}

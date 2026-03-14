use leptos::prelude::*;

// ── Server functions ──────────────────────────────────────────────────────────

/// Request an OTP for the given phone number.
///
/// Always returns `Ok(())` — deliberately does NOT reveal whether a phone is registered.
/// Rate limiting and SMS sending happen inside; errors are swallowed to avoid enumeration.
#[server]
pub async fn request_otp(phone: String) -> Result<(), ServerFnError> {
    use crate::{auth, config::Config, phone as phone_mod, sms};

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;

    // Normalize — on format error, return Ok(()) to avoid revealing anything
    let Ok(normalized) = phone_mod::normalize(&phone) else {
        return Ok(());
    };

    // Check user exists and is active (silently ignore if not)
    let user_exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE phone = $1 AND status = 'active') AS "exists!""#,
        normalized,
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !user_exists {
        return Ok(());
    }

    // Rate limit — silently ignore if exceeded
    if auth::check_otp_rate_limit(&pool, &normalized)
        .await
        .is_err()
    {
        return Ok(());
    }

    // Create OTP code (test mode returns "000000")
    let Ok(code) = auth::create_otp(&pool, &normalized).await else {
        return Ok(());
    };

    // Send SMS (dry-run mode logs instead of sending)
    let message = format!("Ваш код: {code}");
    if let Err(e) = sms::send_sms(&config, &normalized, &message).await {
        tracing::warn!("SMS send failed for {}: {}", normalized, e);
    }

    Ok(())
}

/// Verify an OTP code and set a session cookie on success.
///
/// Returns `true` on successful authentication, `false` on failure.
/// Does NOT return `Err` for wrong code — that is a normal flow.
#[server]
pub async fn verify_otp_code(phone: String, code: String) -> Result<bool, ServerFnError> {
    use crate::{auth, phone as phone_mod};

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;

    let Ok(normalized) = phone_mod::normalize(&phone) else {
        return Ok(false);
    };

    match auth::verify_otp(&pool, &normalized, &code).await {
        Ok((_user_id, raw_token)) => {
            // Set session cookie
            let response_options =
                leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
            let cookie =
                format!("session={raw_token}; HttpOnly; SameSite=Strict; Max-Age=7776000; Path=/");
            response_options.append_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie)
                    .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
            );
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Two-step login: phone input → OTP input.
#[component]
pub fn LoginPage() -> impl IntoView {
    // Step: false = phone entry, true = OTP entry
    let (otp_step, set_otp_step) = signal(false);
    let (phone, set_phone) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let request_action = ServerAction::<RequestOtp>::new();
    let verify_action = ServerAction::<VerifyOtpCode>::new();

    // When request completes, advance to OTP step
    Effect::new(move |_| {
        if let Some(Ok(())) = request_action.value().get() {
            set_otp_step.set(true);
            set_error_msg.set(None);
        }
    });

    // When verify completes, redirect or show error
    Effect::new(move |_| {
        if let Some(result) = verify_action.value().get() {
            match result {
                Ok(true) => {
                    // Redirect — navigate to / and let app guard decide
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/", leptos_router::NavigateOptions::default());
                }
                Ok(false) => {
                    set_error_msg.set(Some("Невірний код або код застарів.".to_owned()));
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Помилка: {e}")));
                }
            }
        }
    });

    view! {
        <div class="login-page">
            <h1>"The Mail Club"</h1>

            {move || if otp_step.get() {
                view! {
                    <form on:submit=move |ev| {
                        ev.prevent_default();
                        verify_action.dispatch(VerifyOtpCode {
                            phone: phone.get(),
                            code: code.get(),
                        });
                    }>
                        <label for="code-input">"SMS code (код з SMS)"</label>
                        <input
                            id="code-input"
                            type="text"
                            name="code"
                            placeholder="000000"
                            maxlength="6"
                            on:input=move |ev| set_code.set(event_target_value(&ev))
                            prop:value=code
                        />
                        <button type="submit">"Verify code (підтвердити)"</button>
                    </form>
                }.into_any()
            } else {
                view! {
                    <form on:submit=move |ev| {
                        ev.prevent_default();
                        request_action.dispatch(RequestOtp { phone: phone.get() });
                    }>
                        <label for="phone-input">"Phone number (номер телефону)"</label>
                        <input
                            id="phone-input"
                            type="tel"
                            name="phone"
                            placeholder="+380XXXXXXXXX"
                            on:input=move |ev| set_phone.set(event_target_value(&ev))
                            prop:value=phone
                        />
                        <button type="submit">"Send code (надіслати)"</button>
                    </form>
                }.into_any()
            }}

            {move || error_msg.get().map(|msg| view! {
                <p class="error">{msg}</p>
            })}
        </div>
    }
}

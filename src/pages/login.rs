use leptos::prelude::*;
use leptos::server_fn::ServerFn;

// ── Server functions ──────────────────────────────────────────────────────────

/// Request an OTP for the given phone number.
///
/// Always returns `Ok(())` — deliberately does NOT reveal whether a phone is registered.
/// Rate limiting and SMS sending happen inside; errors are swallowed to avoid enumeration.
#[server]
pub async fn request_otp(phone: String) -> Result<(), ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        phone as phone_mod, sms,
    };

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
    let prefix = td_string!(Locale::uk, login_otp_sms_body_prefix);
    let message = format!("{prefix}{code}");
    if let Err(e) = sms::send_sms(&config, &normalized, &message).await {
        tracing::warn!("SMS send failed for {}: {}", normalized, e);
    }

    Ok(())
}

/// Verify an OTP code and set a session cookie on success.
///
/// On success: sets a session cookie and redirects to `/`.
/// On failure: redirects back to `/login` — the error is intentionally vague
/// to prevent phone enumeration.
///
/// Uses `leptos_axum::redirect` so the browser follows the 302 — this works
/// for both native form POST (before WASM) and `ActionForm` (after WASM,
/// via `server_fn::redirect` hook).
#[server]
pub async fn verify_otp_code(phone: String, code: String) -> Result<bool, ServerFnError> {
    use crate::{auth, phone as phone_mod};

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;

    let Ok(normalized) = phone_mod::normalize(&phone) else {
        leptos_axum::redirect("/login");
        return Ok(false);
    };

    if let Ok((_user_id, raw_token)) = auth::verify_otp(&pool, &normalized, &code).await {
        // Set session cookie
        let response_options = leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
        let cookie =
            format!("session={raw_token}; HttpOnly; SameSite=Strict; Max-Age=7776000; Path=/");
        response_options.append_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&cookie)
                .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
        );
        leptos_axum::redirect("/");
        Ok(true)
    } else {
        leptos_axum::redirect("/login");
        Ok(false)
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Two-step login: phone input → OTP input.
///
/// Both steps use `ActionForm` which reads form values directly from the DOM
/// at submit time (via `FormData`), bypassing reactive signals entirely.
/// This ensures correct data flow regardless of Leptos hydration timing.
///
/// Before WASM loads: forms submit as native HTML POSTs (progressive enhancement).
/// After WASM hydrates: `ActionForm` intercepts submit and dispatches the action.
#[component]
pub fn LoginPage() -> impl IntoView {
    let request_action = ServerAction::<RequestOtp>::new();

    // Hydration gate — phone form button stays disabled until WASM hydrates.
    // Prevents native form POST (which would show raw JSON) before
    // `ActionForm`'s JS handler is attached.
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

    // Store the phone when it's submitted — capture from the pending input
    // before the action completes and clears `input()`.
    let (submitted_phone, set_submitted_phone) = signal(String::new());
    Effect::new(move |_| {
        if let Some(req) = request_action.input().get() {
            set_submitted_phone.set(req.phone.clone());
        }
    });

    // Show OTP step once request succeeds
    let otp_step = Memo::new(move |_| matches!(request_action.value().get(), Some(Ok(()))));

    view! {
        <div class="prose-page flex flex-col items-center text-center min-h-[80svh] justify-center">
            <img src="/logo.svg" alt="Саме Те · Поштовий клуб" class="h-20 w-auto mb-8" />

            // Phone step — hidden once OTP step activates.
            // Uses ActionForm: reads phone from FormData (DOM) at submit time.
            <div style:display=move || if otp_step.get() { "none" } else { "" }>
                <leptos::form::ActionForm action=request_action>
                    <div class="field">
                        <label class="field-label" for="phone-input">"Phone number (номер телефону)"</label>
                        <input
                            class="field-input"
                            id="phone-input"
                            type="tel"
                            name="phone"
                            placeholder="+380XXXXXXXXX"
                            data-testid="phone-input"
                        />
                    </div>
                    <button class="btn" type="submit" data-testid="send-otp-button" disabled=move || !hydrated.get()>
                        "Send code (надіслати)"
                    </button>
                </leptos::form::ActionForm>
            </div>

            // OTP step — hidden until request succeeds.
            // Uses native form POST: the server sets a cookie and issues a 302
            // redirect. The browser follows the redirect, Playwright waits for
            // the navigation to complete. No WASM involvement needed.
            <div style:display=move || if otp_step.get() { "" } else { "none" }>
                <form method="post" action=VerifyOtpCode::url()>
                    <input type="hidden" name="phone" prop:value=submitted_phone/>
                    <div class="field">
                        <label class="field-label" for="code-input">"SMS code (код з SMS)"</label>
                        <input
                            class="field-input"
                            id="code-input"
                            type="text"
                            name="code"
                            placeholder="000000"
                            maxlength="6"
                            data-testid="otp-input"
                            data-otp
                        />
                    </div>
                    <button class="btn" type="submit" data-testid="verify-otp-button">
                        "Verify code (підтвердити)"
                    </button>
                </form>
            </div>
        </div>
    }
}

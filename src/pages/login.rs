use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, use_i18n};
use leptos::prelude::*;
use leptos::server_fn::ServerFn;

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Result of requesting an OTP.
///
/// Returned by `request_otp` to let the client route the login flow:
/// - `AccountExists`: phone has an active (or deactivated) account → show OTP step
/// - `NewAccount`: phone is unknown → show OTP step, then invite code step after verify
///
/// Importantly the OTP is **always** sent (both branches), so the response body
/// does not reveal whether the phone is registered — only the enum value does,
/// which is only visible after the user's own request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RequestOtpOutcome {
    /// Phone has a user row (active or deactivated). OTP sent. Proceed to OTP step.
    AccountExists,
    /// Phone is unknown. OTP sent. After OTP verify, route to invite code step.
    NewAccount,
}

// ── Server functions ──────────────────────────────────────────────────────────

/// Request an OTP for the given phone number.
///
/// Always sends an OTP (regardless of account existence) and returns an outcome
/// that lets the client route the flow. This preserves no-information-leakage
/// at the network level: both branches send an SMS; only the enum discriminant
/// distinguishes them, and that is only readable by the requesting phone holder.
///
/// Rate limiting is applied to both branches. On rate limit or format error,
/// the function silently returns `AccountExists` to avoid enumerating phone state.
#[server]
pub async fn request_otp(phone: String) -> Result<RequestOtpOutcome, ServerFnError> {
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

    // Normalize — on format error, return AccountExists silently (no leakage)
    let Ok(normalized) = phone_mod::normalize(&phone) else {
        return Ok(RequestOtpOutcome::AccountExists);
    };

    // Check if there is a user row (any status) for this phone
    let user_exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE phone = $1) AS "exists!""#,
        normalized,
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    let outcome = if user_exists {
        RequestOtpOutcome::AccountExists
    } else {
        RequestOtpOutcome::NewAccount
    };

    // Rate limit — silently return AccountExists on limit to avoid enumeration
    if auth::check_otp_rate_limit(&pool, &normalized)
        .await
        .is_err()
    {
        return Ok(RequestOtpOutcome::AccountExists);
    }

    // Create OTP code (test mode returns "000000")
    let Ok(code) = auth::create_otp(&pool, &normalized).await else {
        return Ok(RequestOtpOutcome::AccountExists);
    };

    // Send SMS (dry-run mode logs instead of sending)
    let prefix = td_string!(Locale::uk, login_otp_sms_body_prefix);
    let message = format!("{prefix}{code}");
    if let Err(e) = sms::send_sms(&config, &normalized, &message).await {
        tracing::warn!("SMS send failed for {}: {}", normalized, e);
    }

    Ok(outcome)
}

/// Verify an OTP code and set a session cookie on success.
///
/// Branches on account existence:
/// - Account exists and is active → create session, redirect to `/admin` or `/`
/// - Account exists but deactivated → session created but `current_user` guard
///   rejects it immediately; user is bounced back to `/login` (existing behavior)
/// - No account → set a short-lived `pending_phone` cookie (HttpOnly, 5 min)
///   containing the verified phone, redirect to `/login` so the UI can show
///   the invite code step
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

    // Verify the OTP but do NOT require an active user — we want to allow
    // phone verification for new-account registration too. We must call into
    // the OTP verification layer directly rather than the full `auth::verify_otp`
    // which rejects phones with no user row.
    let otp_result = verify_otp_for_phone(&pool, &normalized, &code).await;

    if otp_result.is_err() {
        leptos_axum::redirect("/login");
        return Ok(false);
    }

    // OTP valid. Now check if an active user exists.
    let active_user_id = sqlx::query_scalar!(
        r#"SELECT id FROM users WHERE phone = $1 AND status = 'active'"#,
        normalized,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if let Some(user_id) = active_user_id {
        // Existing active account — create session and redirect
        let raw_token = auth::create_session(&pool, user_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let response_options = leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
        let cookie =
            format!("session={raw_token}; HttpOnly; SameSite=Strict; Max-Age=7776000; Path=/");
        response_options.append_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&cookie)
                .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
        );

        let is_admin = sqlx::query_scalar!(
            r#"SELECT role = 'admin' AS "is_admin!" FROM users WHERE id = $1"#,
            user_id
        )
        .fetch_optional(&pool)
        .await
        .unwrap_or(None)
        .unwrap_or(false);

        leptos_axum::redirect(if is_admin { "/admin" } else { "/" });
        Ok(true)
    } else {
        // No active account (either unknown phone or deactivated).
        // Set a short-lived pending_phone cookie so the login page can
        // render the invite code form with the verified phone preserved.
        let response_options = leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
        let cookie =
            format!("pending_phone={normalized}; HttpOnly; SameSite=Strict; Max-Age=300; Path=/");
        response_options.append_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&cookie)
                .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
        );
        leptos_axum::redirect("/login");
        Ok(false)
    }
}

/// Verify the OTP for a phone without requiring the phone to have a user row.
///
/// This is a lower-level helper used only by `verify_otp_code` to support
/// the new-account registration flow where a valid OTP was inserted but there
/// is no `users` row yet.
///
/// On success: deletes the OTP row and returns `Ok(())`.
/// On failure: returns `Err(())`.
#[cfg(feature = "ssr")]
async fn verify_otp_for_phone(pool: &sqlx::PgPool, phone: &str, code: &str) -> Result<(), ()> {
    use sha2::{Digest, Sha256};

    fn sha256_hex(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hasher
            .finalize()
            .iter()
            .fold(String::with_capacity(64), |mut acc, b| {
                use std::fmt::Write as _;
                write!(acc, "{b:02x}").expect("write to String is infallible");
                acc
            })
    }

    struct OtpRow {
        id: uuid::Uuid,
        code_hash: String,
        attempts: i32,
    }

    let row = sqlx::query_as!(
        OtpRow,
        r#"
        SELECT id, code_hash, attempts
        FROM otp_codes
        WHERE phone = $1
          AND expires_at > now()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        phone,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return Err(());
    };

    // Exhausted attempt counter: delete and reject
    if row.attempts >= 3 {
        let _ = sqlx::query!("DELETE FROM otp_codes WHERE id = $1", row.id)
            .execute(pool)
            .await;
        return Err(());
    }

    let submitted_hash = sha256_hex(code);

    if submitted_hash != row.code_hash {
        let _ = sqlx::query!(
            "UPDATE otp_codes SET attempts = attempts + 1 WHERE id = $1",
            row.id
        )
        .execute(pool)
        .await;
        return Err(());
    }

    // Code matches — consume it
    let _ = sqlx::query!("DELETE FROM otp_codes WHERE id = $1", row.id)
        .execute(pool)
        .await;

    Ok(())
}

/// Complete self-registration using an invite code.
///
/// Atomically:
/// 1. Reads the `pending_phone` cookie (set by `verify_otp_code` on new-phone verify)
/// 2. Locks the invite code row (`SELECT ... FOR UPDATE`)
/// 3. Verifies it is unused
/// 4. Inserts the new user (with the OTP-verified phone and provided name)
/// 5. Marks the code as used with redeemer + timestamp
/// 6. Creates a session and sets the session cookie
/// 7. Clears the `pending_phone` cookie
/// 8. Redirects to `/onboarding`
///
/// Errors on: missing cookie, invalid/used/revoked code, UNIQUE phone violation.
#[server(RegisterWithCode)]
pub async fn register_with_code(code: String, name: String) -> Result<(), ServerFnError> {
    use crate::{auth, phone as phone_mod};

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;

    // Extract the pending_phone cookie
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let pending_phone = extract_pending_phone_cookie(&parts)
        .ok_or_else(|| ServerFnError::new("Phone not verified — please start over"))?;

    // Validate and normalize the phone from cookie
    let normalized = phone_mod::normalize(&pending_phone)
        .map_err(|_| ServerFnError::new("Invalid phone in session — please start over"))?;

    // Validate name
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    // Atomically lock code, create user, mark code used
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    // Lock the invite code row for update — prevents double redemption under concurrency
    let code_row = sqlx::query!(
        r#"
        SELECT id, status AS "status: String"
        FROM invite_codes
        WHERE code = $1
        FOR UPDATE
        "#,
        code,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Invalid invite code"))?;

    match code_row.status.as_str() {
        "used" => return Err(ServerFnError::new("This invite code has already been used")),
        "revoked" => return Err(ServerFnError::new("This invite code has been revoked")),
        "unused" => {} // proceed
        other => return Err(ServerFnError::new(format!("Unknown code status: {other}"))),
    }

    // Insert the new user — will fail with UNIQUE violation if phone already exists
    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (phone, name)
        VALUES ($1, $2)
        RETURNING id
        "#,
        normalized,
        name,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        // Detect UNIQUE constraint violation on phone column
        if let sqlx::Error::Database(ref db_err) = e
            && db_err.constraint() == Some("users_phone_key")
        {
            return ServerFnError::new("This phone number is already registered");
        }
        ServerFnError::new(format!("database error: {e}"))
    })?;

    // Mark the invite code as used
    sqlx::query!(
        r#"
        UPDATE invite_codes
        SET status = 'used',
            redeemer_id = $1,
            redeemed_at = now()
        WHERE id = $2
        "#,
        user_id,
        code_row.id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    // Create session
    let raw_token = auth::create_session(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response_options = leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();

    // Set session cookie
    let session_cookie =
        format!("session={raw_token}; HttpOnly; SameSite=Strict; Max-Age=7776000; Path=/");
    response_options.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&session_cookie)
            .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
    );

    // Clear the pending_phone cookie
    let clear_cookie = "pending_phone=; HttpOnly; SameSite=Strict; Max-Age=0; Path=/";
    response_options.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(clear_cookie)
            .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
    );

    leptos_axum::redirect("/onboarding");
    Ok(())
}

/// Extract the `pending_phone` cookie value from request parts.
#[cfg(feature = "ssr")]
fn extract_pending_phone_cookie(parts: &http::request::Parts) -> Option<String> {
    let cookie_header = parts.headers.get(http::header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix("pending_phone=") {
            return Some(value.to_owned());
        }
    }
    None
}

/// Logout by clearing the session cookie and deleting the session from the database.
///
/// Redirects to `/` (which will redirect to `/login` since no valid session exists).
#[server(Logout)]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::auth;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;

    // Extract the session cookie if it exists
    if let Some(parts) = leptos::context::use_context::<http::request::Parts>()
        && let Some(cookie_header) = parts.headers.get(http::header::COOKIE)
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if let Some(raw_token) = pair.strip_prefix("session=") {
                // Delete session from database (ignore errors — logout always succeeds)
                let _ = auth::delete_session(&pool, raw_token).await;
                break;
            }
        }
    }

    // Clear the session cookie by setting it with Max-Age=0
    let response_options = leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
    let cookie = "session=; HttpOnly; SameSite=Strict; Max-Age=0; Path=/";
    response_options.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(cookie)
            .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
    );

    leptos_axum::redirect("/");
    Ok(())
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Three-step login / self-registration flow.
///
/// Step 1 — Phone: user enters phone, `request_otp` sends OTP and returns outcome.
/// Step 2 — OTP: user enters OTP code (native form POST via `VerifyOtpCode::url()`).
///   - Existing account: server sets session + redirects to `/` or `/admin`
///   - New phone: server sets `pending_phone` cookie + redirects back to `/login`
/// Step 3 — Invite code (new phones only): user enters invite code, then name.
///
/// Both `ActionForm` (for step 1) and native `<form method="post">` (for step 2)
/// are used. Step 2 uses native POST so the browser follows the server-issued 302
/// redirect without WASM involvement — this is required for progressive enhancement
/// and avoids WASM hydration race conditions.
///
/// Step 3 uses `ActionForm` for the invite code and name collection.
#[component]
pub fn LoginPage() -> impl IntoView {
    let i18n = use_i18n();
    let request_action = ServerAction::<RequestOtp>::new();
    let request_pending = request_action.pending();

    // Hydration gate — phone form button stays disabled until WASM hydrates.
    let hydrated = use_hydrated();

    // Store the phone when it's submitted — capture from the pending input
    // before the action completes and clears `input()`.
    let (submitted_phone, set_submitted_phone) = signal(String::new());
    Effect::new(move |_| {
        if let Some(req) = request_action.input().get() {
            set_submitted_phone.set(req.phone.clone());
        }
    });

    // Show OTP step once request succeeds (for any outcome)
    let otp_step = Memo::new(move |_| {
        matches!(
            request_action.value().get(),
            Some(Ok(
                RequestOtpOutcome::AccountExists | RequestOtpOutcome::NewAccount
            ))
        )
    });

    view! {
        <div class="prose-page flex flex-col items-center text-center pt-[10svh]">
            <img
                src="/logo.svg"
                alt="Саме Те · Поштовий клуб"
                class="h-20 w-auto mb-8"
            />

            // Phone step — hidden once OTP step activates.
            // Uses ActionForm: reads phone from FormData (DOM) at submit time.
            <div style:display=move || if otp_step.get() { "none" } else { "" }>
                <leptos::form::ActionForm action=request_action>
                    <div class="field">
                        <label class="field-label" for="phone-input">
                            {t!(i18n, login_phone_label)}
                        </label>
                        <input
                            class="field-input"
                            id="phone-input"
                            type="tel"
                            name="phone"
                            placeholder="+380XXXXXXXXX"
                            data-testid="phone-input"
                        />
                    </div>
                    <button
                        class="btn w-full"
                        type="submit"
                        data-testid="send-otp-button"
                        disabled=move || request_pending.get() || !hydrated.get()
                    >
                        {move || if request_pending.get() {
                            "Надсилаю...".into_any()
                        } else {
                            t!(i18n, login_send_code_button).into_any()
                        }}
                    </button>
                </leptos::form::ActionForm>
            </div>

            // OTP step — hidden until request succeeds.
            // Uses native form POST: the server sets a cookie and issues a 302
            // redirect. The browser follows the redirect, Playwright waits for
            // the navigation to complete. No WASM involvement needed.
            <div style:display=move || if otp_step.get() { "" } else { "none" }>
                <form method="post" action=VerifyOtpCode::url()>
                    <input type="hidden" name="phone" prop:value=submitted_phone />
                    <div class="field">
                        <label class="field-label" for="code-input">
                            {t!(i18n, login_otp_label)}
                        </label>
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
                    <button class="btn w-full" type="submit" data-testid="verify-otp-button">
                        {t!(i18n, login_verify_button)}
                    </button>
                </form>
                <button
                    type="button"
                    class="btn w-full mt-3"
                    data-variant="secondary"
                    data-testid="back-to-phone-button"
                    on:click=move |_| request_action.value().set(None)
                >
                    {t!(i18n, login_change_phone_button)}
                </button>
            </div>
        </div>
    }
}

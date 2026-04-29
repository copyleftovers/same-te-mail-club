use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, t_string, use_i18n};
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
/// - Account exists but deactivated → redirect to `/login` silently (no cookie set).
///   The user sees the login page with no indication of why — prevents phone enumeration
///   from the invite-code registration path.
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

    // OTP valid. Now check for any existing user row (any status) for this phone.
    // We must distinguish three cases:
    //   1. Active user → create session, redirect to home/admin
    //   2. Deactivated user → redirect to /login without setting pending_phone cookie
    //      (prevents leaking that the phone exists via the invite-code error path)
    //   3. Unknown phone → set pending_phone cookie, redirect to /login for invite step
    let existing_status: Option<String> = sqlx::query_scalar!(
        r#"SELECT status::TEXT FROM users WHERE phone = $1"#,
        normalized,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .flatten();

    match existing_status.as_deref() {
        Some("active") => {
            // Existing active account — create session and redirect
            let user_id = sqlx::query_scalar!(
                r#"SELECT id FROM users WHERE phone = $1 AND status = 'active'"#,
                normalized,
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

            let raw_token = auth::create_session(&pool, user_id)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            let response_options =
                leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
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
        }
        Some(_) => {
            // Deactivated user — redirect silently without setting pending_phone.
            // This prevents a deactivated phone from entering the invite-code
            // registration path, which would hit a UNIQUE constraint and reveal
            // that the phone was already registered.
            leptos_axum::redirect("/login");
            Ok(false)
        }
        None => {
            // Unknown phone — set pending_phone HttpOnly cookie (server-side auth token
            // for validate_invite_code and register_with_code) AND redirect to
            // /login?pending=1 (client-side signal for UI routing). Dual mechanism:
            // cookie = security, query param = UI.
            let response_options =
                leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
            let cookie = format!(
                "pending_phone={normalized}; HttpOnly; SameSite=Strict; Max-Age=300; Path=/"
            );
            response_options.append_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie)
                    .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
            );
            leptos_axum::redirect("/login?pending=1");
            Ok(false)
        }
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

/// Validate an invite code without redeeming it.
///
/// Checks that the `pending_phone` cookie exists (phone was OTP-verified) and
/// that the given code exists in `invite_codes` with status `'unused'`.
///
/// This is an advisory check — it provides early UX feedback so the user
/// discovers a bad code at the code step rather than after entering their name.
/// The atomic redemption in `register_with_code` is the real gate.
///
/// Returns the validated code string on success so the caller can pass it to
/// the name step without re-reading the form.
///
/// # Errors
///
/// Returns `Err` if:
/// - the `pending_phone` cookie is absent (phone not OTP-verified)
/// - the code is empty
/// - no `invite_codes` row matches the code
/// - the matching code is not in `'unused'` status
#[server(ValidateInviteCode)]
pub async fn validate_invite_code(code: String) -> Result<String, ServerFnError> {
    use crate::types::InviteCodeStatus;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;

    // Require that the phone was OTP-verified (pending_phone cookie present)
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;
    if extract_pending_phone_cookie(&parts).is_none() {
        return Err(ServerFnError::new("Phone not verified — please start over"));
    }

    let code = code.trim().to_owned();
    if code.is_empty() {
        return Err(ServerFnError::new("Invite code is required"));
    }

    let status: Option<InviteCodeStatus> = sqlx::query_scalar!(
        r#"SELECT status AS "status: InviteCodeStatus" FROM invite_codes WHERE code = $1"#,
        code,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    match status {
        None => Err(ServerFnError::new("Invalid invite code")),
        Some(InviteCodeStatus::Used) => {
            Err(ServerFnError::new("This invite code has already been used"))
        }
        Some(InviteCodeStatus::Revoked) => {
            Err(ServerFnError::new("This invite code has been revoked"))
        }
        Some(InviteCodeStatus::Unused) => Ok(code),
    }
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

/// Check whether the current request has a `pending_phone` cookie set,
/// indicating the user has verified their phone but not yet completed registration.
///
/// This is called as a server Resource on page load so the login page can
/// render the correct initial step during SSR.
#[server(CheckPendingRegistration)]
// `async` is required by the `#[server]` macro even though this function has no
// await points — the macro generates an async trait impl that requires it.
#[allow(clippy::unused_async)]
pub async fn check_pending_registration() -> Result<bool, ServerFnError> {
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;
    Ok(extract_pending_phone_cookie(&parts).is_some())
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

/// Four-step login / self-registration flow.
///
/// Step 1 — Phone: user enters phone, `request_otp` sends OTP and returns outcome.
/// Step 2 — OTP: user enters OTP code (native form POST via `VerifyOtpCode::url()`).
///   - Existing account: server sets session + redirects to `/` or `/admin`
///   - New phone: server sets `pending_phone` cookie + redirects back to `/login`
/// Step 3 — Invite code (new phones only): user enters invite code; on submit,
///   the code is stored client-side and the name step appears.
/// Step 4 — Name (new phones only): user enters their full name; `RegisterWithCode`
///   ActionForm sends both code + name, server creates the account and redirects
///   to `/onboarding`.
///
/// Step visibility is controlled by `style:display` toggling. The four steps are
/// mutually exclusive and never unmounted — toggling avoids DOM churn and maintains
/// form state across re-renders.
///
/// Cookie detection for step 3/4 is done via a server Resource (`check_pending_registration`)
/// called on page load. This works during SSR and after hydration.
#[component]
pub fn LoginPage() -> impl IntoView {
    let request_action = ServerAction::<RequestOtp>::new();

    // Hydration gate — buttons stay disabled until WASM hydrates.
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

    // Detect ?pending=1 query param (set by verify_otp_code for new phones).
    // Must produce the same result on SSR and client for hydration stability.
    // The actual phone is in an HttpOnly cookie; this param just signals the UI.
    let is_pending = {
        #[cfg(feature = "ssr")]
        {
            leptos::context::use_context::<http::request::Parts>()
                .and_then(|parts| parts.uri.query().map(|q| q.contains("pending=1")))
                .unwrap_or(false)
        }
        #[cfg(not(feature = "ssr"))]
        {
            leptos::prelude::window()
                .location()
                .search()
                .ok()
                .is_some_and(|s| s.contains("pending=1"))
        }
    };
    let (is_pending_signal, _) = signal(is_pending);

    // Client-side signal: the invite code entered in step 3.
    // None = step 3 shown; Some(code) = step 4 shown.
    let (entered_code, set_entered_code) = signal(Option::<String>::None);

    // ActionForm for name collection step (step 4)
    let register_action = ServerAction::<RegisterWithCode>::new();

    view! {
        <div class="prose-page flex flex-col items-center text-center pt-[10svh]">
            <img
                src="/logo.svg"
                alt="Саме Те · Поштовий клуб"
                class="h-20 w-auto mb-8"
            />

            <LoginStepRouter
                is_pending=is_pending_signal.get()
                otp_step=otp_step
                hydrated=hydrated
                request_action=request_action
                submitted_phone=submitted_phone
                entered_code=entered_code
                set_entered_code=set_entered_code
                register_action=register_action
            />
        </div>
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

/// Routes between the four login/registration steps based on current state.
///
/// All steps are rendered but toggled visible/hidden via `style:display` so
/// form state is preserved across transitions.
///
/// Step visibility rules:
/// - Step 1 (phone):        `!otp_step && !is_pending`
/// - Step 2 (OTP):          `otp_step && !is_pending`
/// - Step 3 (invite code):  `is_pending && entered_code.is_none()`
/// - Step 4 (name):         `is_pending && entered_code.is_some()`
#[allow(clippy::too_many_arguments)]
// This component coordinates four steps with their shared signals. The argument
// count reflects the necessary state threads, not a design smell; splitting would
// require context or channels instead, which adds indirection for no clarity gain.
#[component]
fn LoginStepRouter(
    is_pending: bool,
    otp_step: Memo<bool>,
    hydrated: ReadSignal<bool>,
    request_action: ServerAction<RequestOtp>,
    submitted_phone: ReadSignal<String>,
    entered_code: ReadSignal<Option<String>>,
    set_entered_code: WriteSignal<Option<String>>,
    register_action: ServerAction<RegisterWithCode>,
) -> impl IntoView {
    let i18n = use_i18n();
    let request_pending = request_action.pending();

    view! {
        // ── Step 1: Phone ─────────────────────────────────────────────────────
        // Hidden once OTP step activates or pending_registration is set.
        <div style:display=move || {
            if otp_step.get() || is_pending { "none" } else { "" }
        }>
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
                        t!(i18n, common_loading).into_any()
                    } else {
                        t!(i18n, login_send_code_button).into_any()
                    }}
                </button>
            </leptos::form::ActionForm>
        </div>

        // ── Step 2: OTP ───────────────────────────────────────────────────────
        // Hidden until request succeeds; also hidden if pending_registration is set.
        // Uses native form POST so the browser follows the server-issued 302 redirect.
        <div style:display=move || {
            if otp_step.get() && !is_pending { "" } else { "none" }
        }>
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

        // ── Step 3: Invite code ───────────────────────────────────────────────
        // Shown when pending_registration is true AND no code entered yet.
        <div
            data-testid="invite-code-step"
            style:display=move || {
                if is_pending && entered_code.get().is_none() { "" } else { "none" }
            }
        >
            <h2 class="font-display text-xl font-black mb-4">
                {t!(i18n, auth_invite_code_step_heading)}
            </h2>
            <InviteCodeForm
                hydrated=hydrated
                on_submit=move |code| set_entered_code.set(Some(code))
            />
        </div>

        // ── Step 4: Name collection ────────────────────────────────────────────
        // Shown when pending_registration is true AND a code has been entered.
        <div
            data-testid="name-collection-step"
            style:display=move || {
                if is_pending && entered_code.get().is_some() { "" } else { "none" }
            }
        >
            <NameCollectionForm
                hydrated=hydrated
                register_action=register_action
                entered_code=entered_code
                on_back=move || set_entered_code.set(None)
            />
        </div>
    }
}

/// Invite code entry form (step 3).
///
/// Calls `validate_invite_code` via `ActionForm` to check the code on the server
/// before advancing to the name step. This surfaces "invalid / used / revoked"
/// errors at the code step rather than after the user fills in their name.
///
/// On successful validation, calls `on_submit` with the validated code string and
/// transitions to step 4. On failure, displays the server error inline.
///
/// Note: `validate_invite_code` is advisory. `register_with_code` performs the
/// atomic redemption and is the real correctness gate.
#[component]
fn InviteCodeForm<F>(hydrated: ReadSignal<bool>, on_submit: F) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    let i18n = use_i18n();
    let validate_action = ServerAction::<ValidateInviteCode>::new();
    let validate_pending = validate_action.pending();

    // When the action succeeds, forward the validated code to the parent.
    Effect::new(move |_| {
        if let Some(Ok(ref code)) = validate_action.value().get() {
            on_submit(code.clone());
        }
    });

    let error_msg = move || {
        validate_action
            .value()
            .get()
            .and_then(Result::err)
            .map(|e| e.to_string())
    };

    view! {
        <leptos::form::ActionForm action=validate_action>
            <div class="field">
                <label class="field-label" for="invite-code-input">
                    {t!(i18n, auth_invite_code_prompt)}
                </label>
                <input
                    class="field-input"
                    id="invite-code-input"
                    type="text"
                    name="code"
                    placeholder=move || t_string!(i18n, auth_invite_code_placeholder)
                    data-testid="invite-code-input"
                    aria-describedby="invite-code-error"
                    attr:aria-invalid=move || error_msg().is_some().then_some("true")
                />
            </div>
            // Error display for invite code step — server-returned message
            <p
                id="invite-code-error"
                class="text-sm text-(--color-error) mt-1 mb-3"
                aria-live="assertive"
                data-testid="invite-code-error"
            >
                {move || error_msg()}
            </p>
            <button
                class="btn w-full"
                type="submit"
                data-testid="submit-invite-code-button"
                disabled=move || validate_pending.get() || !hydrated.get()
            >
                {move || if validate_pending.get() {
                    t!(i18n, common_loading).into_any()
                } else {
                    t!(i18n, auth_invite_code_submit_button).into_any()
                }}
            </button>
        </leptos::form::ActionForm>
    }
}

/// Name collection form (step 4 of the registration flow).
///
/// Uses `ActionForm` to call `RegisterWithCode` with a hidden code field (from
/// the previous step) plus the user-provided name. On success, the server redirects
/// to `/onboarding`. On error, the error is displayed inline.
///
/// The `on_back` callback lets the user return to the invite code step to re-enter
/// the code if they made a typo.
#[component]
fn NameCollectionForm<F>(
    hydrated: ReadSignal<bool>,
    register_action: ServerAction<RegisterWithCode>,
    entered_code: ReadSignal<Option<String>>,
    on_back: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let i18n = use_i18n();
    let register_pending = register_action.pending();

    view! {
        <h2 class="font-display text-xl font-black mb-4">
            {t!(i18n, auth_name_step_heading)}
        </h2>
        <p class="text-sm text-(--color-text-muted) mb-4">
            {t!(i18n, auth_name_context)}
        </p>
        <leptos::form::ActionForm action=register_action>
            // Hidden input carries the code from step 3 into the form submission
            <input
                type="hidden"
                name="code"
                prop:value=move || entered_code.get().unwrap_or_default()
            />
            <div class="field">
                <label class="field-label" for="legal-name-input">
                    {t!(i18n, auth_name_prompt)}
                </label>
                <input
                    class="field-input"
                    id="legal-name-input"
                    type="text"
                    name="name"
                    placeholder=move || t_string!(i18n, participants_name_placeholder)
                    data-testid="legal-name-input"
                    aria-describedby="legal-name-error"
                    attr:aria-invalid=move || {
                        register_action
                            .value()
                            .get()
                            .and_then(Result::err)
                            .is_some()
                            .then_some("true")
                    }
                />
            </div>
            // Error display for name collection
            <p
                id="legal-name-error"
                class="text-sm text-(--color-error) mt-1 mb-3"
                aria-live="assertive"
                data-testid="legal-name-error"
            >
                {move || register_action.value().get()
                    .and_then(Result::err)
                    .map(|e| e.to_string())}
            </p>
            <button
                class="btn w-full"
                type="submit"
                data-testid="create-account-button"
                disabled=move || register_pending.get() || !hydrated.get()
            >
                {move || if register_pending.get() {
                    t!(i18n, common_loading).into_any()
                } else {
                    t!(i18n, auth_name_submit_button).into_any()
                }}
            </button>
        </leptos::form::ActionForm>
        // Back button — lets user re-enter the code if they made a typo
        <button
            type="button"
            class="btn w-full mt-3"
            data-variant="secondary"
            data-testid="back-to-invite-code-button"
            on:click=move |_| on_back()
        >
            {t!(i18n, login_change_phone_button)}
        </button>
    }
}

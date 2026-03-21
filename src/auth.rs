use crate::{
    error::AppError,
    types::{UserRole, UserStatus},
};
use base64::Engine as _;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

// ── Private row types (only used inside this module) ─────────────────────────

struct OtpRow {
    id: Uuid,
    code_hash: String,
    attempts: i32,
}

struct SessionRow {
    user_id: Uuid,
    expires_at: time::OffsetDateTime,
}

struct UserRow {
    name: String,
    role: UserRole,
    status: UserStatus,
    onboarded: bool,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

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

fn extract_session_cookie(parts: &http::request::Parts) -> Option<String> {
    let cookie_header = parts.headers.get(http::header::COOKIE)?.to_str().ok()?;
    // Parse "session=<token>" from the Cookie header (multiple cookies possible)
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix("session=") {
            return Some(value.to_owned());
        }
    }
    None
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate a 6-digit OTP, hash it, INSERT into `otp_codes`.
///
/// Old codes are retained (not upserted) to enable rate-limit row counting.
/// Returns the raw code string (for SMS delivery).
///
/// When `SAMETE_TEST_MODE=true`, always returns `"000000"`.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn create_otp(pool: &PgPool, phone: &str) -> Result<String, AppError> {
    let code = if std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true") {
        "000000".to_owned()
    } else {
        // Use rand to generate a 6-digit code, zero-padded
        use rand::Rng as _;
        format!("{:06}", rand::rng().random_range(0u32..1_000_000))
    };

    let code_hash = sha256_hex(&code);

    sqlx::query!(
        r#"
        INSERT INTO otp_codes (phone, code_hash, expires_at)
        VALUES ($1, $2, now() + interval '10 minutes')
        "#,
        phone,
        code_hash,
    )
    .execute(pool)
    .await?;

    Ok(code)
}

/// Check OTP rate limits by counting existing rows (no in-memory state).
///
/// Two tiers:
/// - Max 1 OTP per 60 seconds per phone
/// - Max 5 OTPs per hour per phone
///
/// # Errors
///
/// Returns `Err(AppError::RateLimited)` if either limit is exceeded.
/// Returns `Err(AppError::Database(_))` on DB failure.
pub async fn check_otp_rate_limit(pool: &PgPool, phone: &str) -> Result<(), AppError> {
    if std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true") {
        return Ok(());
    }

    let count_60s = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM otp_codes
        WHERE phone = $1
          AND created_at > now() - interval '60 seconds'
        "#,
        phone,
    )
    .fetch_one(pool)
    .await?;

    if count_60s >= 1 {
        return Err(AppError::RateLimited);
    }

    let count_1h = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM otp_codes
        WHERE phone = $1
          AND created_at > now() - interval '1 hour'
        "#,
        phone,
    )
    .fetch_one(pool)
    .await?;

    if count_1h >= 5 {
        return Err(AppError::RateLimited);
    }

    Ok(())
}

/// Verify an OTP code against the stored hash.
///
/// Selects the most recent non-expired code for the phone.
/// On success: deletes the OTP row, looks up the user, creates a session,
/// and returns `(user_id, session_token)`.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if no valid code exists, the code is
/// wrong, attempts are exhausted, or the user is not found / not active.
pub async fn verify_otp(
    pool: &PgPool,
    phone: &str,
    code: &str,
) -> Result<(Uuid, String), AppError> {
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
    .await?;

    let row = row.ok_or(AppError::Unauthorized)?;

    // Exhausted attempt counter: delete and reject
    if row.attempts >= 3 {
        sqlx::query!("DELETE FROM otp_codes WHERE id = $1", row.id)
            .execute(pool)
            .await?;
        return Err(AppError::Unauthorized);
    }

    let submitted_hash = sha256_hex(code);

    if submitted_hash != row.code_hash {
        sqlx::query!(
            "UPDATE otp_codes SET attempts = attempts + 1 WHERE id = $1",
            row.id
        )
        .execute(pool)
        .await?;
        return Err(AppError::Unauthorized);
    }

    // Code matches — consume it
    sqlx::query!("DELETE FROM otp_codes WHERE id = $1", row.id)
        .execute(pool)
        .await?;

    // Look up active user by phone
    let user_id = sqlx::query_scalar!(
        r#"SELECT id FROM users WHERE phone = $1 AND status = 'active'"#,
        phone,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let token = create_session(pool, user_id).await?;
    Ok((user_id, token))
}

/// Create a new session and return the raw (unhashed) token for cookie use.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<String, AppError> {
    use rand::RngCore as _;
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let raw_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let token_hash = sha256_hex(&raw_token);

    sqlx::query!(
        r#"
        INSERT INTO sessions (token_hash, user_id, expires_at)
        VALUES ($1, $2, now() + interval '90 days')
        "#,
        token_hash,
        user_id,
    )
    .execute(pool)
    .await?;

    Ok(raw_token)
}

/// Validate a session from the raw cookie token value.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if the session is not found or expired.
pub async fn validate_session(pool: &PgPool, raw_token: &str) -> Result<Uuid, AppError> {
    let token_hash = sha256_hex(raw_token);

    let row = sqlx::query_as!(
        SessionRow,
        r#"
        SELECT user_id, expires_at
        FROM sessions
        WHERE token_hash = $1
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or(AppError::Unauthorized)?;

    let now = time::OffsetDateTime::now_utc();
    if row.expires_at < now {
        sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
            .execute(pool)
            .await?;
        return Err(AppError::Unauthorized);
    }

    Ok(row.user_id)
}

/// Delete a session (logout).
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn delete_session(pool: &PgPool, raw_token: &str) -> Result<(), AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get the current user from request cookies.
///
/// Used by server functions and route guards.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if no valid session exists or the user is inactive.
pub async fn current_user(
    pool: &PgPool,
    parts: &http::request::Parts,
) -> Result<CurrentUser, AppError> {
    let raw_token = extract_session_cookie(parts).ok_or(AppError::Unauthorized)?;
    let user_id = validate_session(pool, &raw_token).await?;

    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT
            name,
            role AS "role: UserRole",
            status AS "status: UserStatus",
            onboarded
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if row.status != UserStatus::Active {
        return Err(AppError::Unauthorized);
    }

    Ok(CurrentUser {
        id: user_id,
        name: row.name,
        role: row.role,
        onboarded: row.onboarded,
    })
}

// `CurrentUser` lives in `crate::types` so it's available in both SSR and WASM builds.
pub use crate::types::CurrentUser;

// ── Server function context helpers ─────────────────────────────────────────

/// Extract the database pool and authenticated user from server function context.
///
/// Combines context extraction + session validation into one call.
///
/// # Errors
///
/// Returns `Err(ServerFnError)` if context is missing or session is invalid.
pub async fn require_auth() -> Result<(PgPool, CurrentUser), leptos::prelude::ServerFnError> {
    let pool = leptos::context::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("no request parts in context"))?;

    let user = current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    Ok((pool, user))
}

/// Extract the database pool and verify the caller is an admin.
///
/// # Errors
///
/// Returns `Err(ServerFnError)` if not authenticated or not an admin.
pub async fn require_admin() -> Result<(PgPool, CurrentUser), leptos::prelude::ServerFnError> {
    let (pool, user) = require_auth().await?;

    if user.role != crate::types::UserRole::Admin {
        return Err(AppError::Forbidden.into_server_fn_error());
    }

    Ok((pool, user))
}

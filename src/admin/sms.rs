use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Summary of an SMS batch send operation.
///
/// Returned to the admin after triggering any SMS batch.
/// `sent` + `failed` = total recipients targeted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsReport {
    pub sent: u32,
    pub failed: u32,
    pub failed_phones: Vec<String>,
}

/// SSR-only row type for notification targets identified by a single UUID.
/// Used by assignment SMS (id = assignment.id) and receipt nudge (id = assignment.id).
#[cfg(feature = "ssr")]
struct AssignmentTarget {
    id: uuid::Uuid,
    phone: String,
}

/// SSR-only row type for season-open notification targets.
/// `user_id` is needed to insert into `season_open_notifications` on success.
#[cfg(feature = "ssr")]
struct SeasonOpenTarget {
    user_id: uuid::Uuid,
    phone: String,
}

/// SSR-only row type for confirm-nudge notification targets.
/// `user_id` is the enrollment FK needed to update `confirm_nudge_sent_at`.
#[cfg(feature = "ssr")]
struct ConfirmNudgeTarget {
    user_id: uuid::Uuid,
    phone: String,
}

// ── SMS target count helpers ──────────────────────────────────────────────────
//
// Each helper encodes the exact WHERE predicate shared between the corresponding
// `send_*_sms` function's target query and the admin state count display.
// `state.rs::query_sms_counts` calls these so the predicates have one home.

/// Count active participants not yet notified of the season opening.
///
/// Matches the target predicate of [`send_season_open_sms`].
#[cfg(feature = "ssr")]
pub(super) async fn count_season_open_targets(
    pool: &sqlx::PgPool,
    season_id: uuid::Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM users u
        LEFT JOIN season_open_notifications son
            ON son.user_id = u.id AND son.season_id = $1
        WHERE u.status = 'active' AND u.role = 'participant'
          AND son.user_id IS NULL
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
}

/// Count assignments whose senders have not been notified yet.
///
/// Matches the target predicate of [`send_assignment_sms`].
#[cfg(feature = "ssr")]
pub(super) async fn count_unnotified_senders(
    pool: &sqlx::PgPool,
    season_id: uuid::Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM assignments a
        WHERE a.season_id = $1 AND a.notified_at IS NULL
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
}

/// Count enrolled users not yet nudged to confirm readiness.
///
/// Matches the target predicate of [`send_confirm_nudge_sms`].
#[cfg(feature = "ssr")]
pub(super) async fn count_unconfirmed_enrolled(
    pool: &sqlx::PgPool,
    season_id: uuid::Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM enrollments e
        WHERE e.season_id = $1
          AND e.confirmed_ready_at IS NULL
          AND e.confirm_nudge_sent_at IS NULL
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
}

/// Count recipients with no response who have not yet been nudged.
///
/// Matches the target predicate of [`send_receipt_nudge_sms`].
#[cfg(feature = "ssr")]
pub(super) async fn count_no_response_recipients(
    pool: &sqlx::PgPool,
    season_id: uuid::Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM assignments a
        WHERE a.season_id = $1
          AND a.receipt_status = 'no_response'
          AND a.receipt_nudge_sent_at IS NULL
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
}

// ── Server functions ───────────────────────────────────────────────────────────

/// Story 5.3: Season-open notification to all active participants.
///
/// Sent when the organizer launches a season. Targets every active participant
/// (excludes admin accounts) who hasn't been notified yet for this season.
/// On success, inserts a row into `season_open_notifications`; on failure,
/// leaves no row so the admin can re-trigger.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_season_open_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;
    let http_client = leptos::context::use_context::<reqwest::Client>()
        .ok_or_else(|| ServerFnError::new("no HTTP client in context"))?;

    let season_id = super::db_helpers::fetch_active_launched_season(&pool)
        .await
        .map_err(db_err)?
        .ok_or_else(|| ServerFnError::new(td_string!(Locale::uk, season_error_no_launched_season)))?
        .id;

    let targets = sqlx::query_as!(
        SeasonOpenTarget,
        r#"
        SELECT u.id AS user_id, u.phone
        FROM users u
        LEFT JOIN season_open_notifications son
            ON son.user_id = u.id AND son.season_id = $1
        WHERE u.status = 'active' AND u.role = 'participant'
          AND son.user_id IS NULL
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let message = td_string!(Locale::uk, sms_season_open_body);

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for target in targets {
        match sms::send_sms(&config, &http_client, &target.phone, message).await {
            Ok(()) => {
                let _ = sqlx::query!(
                    r#"INSERT INTO season_open_notifications (user_id, season_id) VALUES ($1, $2) ON CONFLICT (user_id, season_id) DO NOTHING"#,
                    target.user_id,
                    season_id,
                )
                .execute(&pool)
                .await;
                sent += 1;
            }
            Err(e) => {
                tracing::warn!(phone = %target.phone, error = %e, "SMS send failed");
                failed_phones.push(target.phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

/// Story 5.1: Assignment notification — sets `notified_at` per-assignment on success.
///
/// Sent to senders whose `notified_at IS NULL` (haven't been notified yet).
/// On failure, leaves `notified_at` NULL so the organizer can see who wasn't reached.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_assignment_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;
    let http_client = leptos::context::use_context::<reqwest::Client>()
        .ok_or_else(|| ServerFnError::new("no HTTP client in context"))?;

    // Intentionally narrower than the canonical active-launched predicate: requires
    // phase IN ('assignment','delivery') so a season without assignments yet (e.g.
    // 'preparation') produces a clear error instead of an empty send loop.
    let season_id = sqlx::query_scalar!(
        r#"SELECT id FROM seasons WHERE phase IN ('assignment', 'delivery') AND launched_at IS NOT NULL"#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| {
        ServerFnError::new(td_string!(
            Locale::uk,
            sms_error_no_assignment_delivery_season
        ))
    })?;

    // Senders who haven't been notified yet
    let targets = sqlx::query_as!(
        AssignmentTarget,
        r#"
        SELECT a.id, u.phone
        FROM assignments a
        JOIN users u ON u.id = a.sender_id
        WHERE a.season_id = $1 AND a.notified_at IS NULL
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let message = td_string!(Locale::uk, sms_assignment_body);

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for target in targets {
        match sms::send_sms(&config, &http_client, &target.phone, message).await {
            Ok(()) => {
                // Mark notified_at on success; leave NULL on failure
                let _ = sqlx::query!(
                    r#"UPDATE assignments SET notified_at = now() WHERE id = $1"#,
                    target.id,
                )
                .execute(&pool)
                .await;
                sent += 1;
            }
            Err(e) => {
                tracing::warn!(phone = %target.phone, error = %e, "SMS send failed");
                failed_phones.push(target.phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

/// Story 5.4: Pre-deadline nudge to enrolled users who haven't confirmed ready.
///
/// Targets enrolled users with `confirmed_ready_at IS NULL` and
/// `confirm_nudge_sent_at IS NULL` (haven't been nudged yet).
/// On success, sets `confirm_nudge_sent_at`; on failure, leaves NULL
/// so the admin can re-trigger.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_confirm_nudge_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;
    let http_client = leptos::context::use_context::<reqwest::Client>()
        .ok_or_else(|| ServerFnError::new("no HTTP client in context"))?;

    // Phase-specific: pre-deadline SMS is only valid during 'preparation'.
    // Intentionally narrower than the canonical active-launched predicate.
    let season_id = sqlx::query_scalar!(
        r#"SELECT id FROM seasons WHERE phase = 'preparation' AND launched_at IS NOT NULL"#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new(td_string!(Locale::uk, sms_error_no_preparation_season)))?;

    let confirm_deadline = sqlx::query_scalar!(
        r#"SELECT confirm_deadline FROM seasons WHERE id = $1"#,
        season_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    let deadline_str = crate::date_format::format_date_uk(confirm_deadline);

    let targets = sqlx::query_as!(
        ConfirmNudgeTarget,
        r#"
        SELECT e.user_id, u.phone
        FROM enrollments e
        JOIN users u ON u.id = e.user_id
        WHERE e.season_id = $1
          AND e.confirmed_ready_at IS NULL
          AND e.confirm_nudge_sent_at IS NULL
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let prefix = td_string!(Locale::uk, sms_confirm_nudge_body_prefix);
    let message = format!("{prefix}{deadline_str}.");

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for target in targets {
        match sms::send_sms(&config, &http_client, &target.phone, &message).await {
            Ok(()) => {
                let _ = sqlx::query!(
                    r#"UPDATE enrollments SET confirm_nudge_sent_at = now() WHERE user_id = $1 AND season_id = $2"#,
                    target.user_id,
                    season_id,
                )
                .execute(&pool)
                .await;
                sent += 1;
            }
            Err(e) => {
                tracing::warn!(phone = %target.phone, error = %e, "SMS send failed");
                failed_phones.push(target.phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

/// Story 5.2: Receipt nudge to recipients with `receipt_status = 'no_response'`.
///
/// Sent after assignments have been delivered to prompt confirmation.
/// Targets recipients whose `receipt_nudge_sent_at IS NULL` (haven't been nudged yet).
/// On success, sets `receipt_nudge_sent_at`; on failure, leaves NULL
/// so the admin can re-trigger.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_receipt_nudge_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;
    let http_client = leptos::context::use_context::<reqwest::Client>()
        .ok_or_else(|| ServerFnError::new("no HTTP client in context"))?;

    // Phase-specific: receipt nudge SMS is only valid during 'delivery'.
    // Intentionally narrower than the canonical active-launched predicate.
    let season_id = sqlx::query_scalar!(
        r#"SELECT id FROM seasons WHERE phase = 'delivery' AND launched_at IS NOT NULL"#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new(td_string!(Locale::uk, sms_error_no_delivery_season)))?;

    let targets = sqlx::query_as!(
        AssignmentTarget,
        r#"
        SELECT a.id, u.phone
        FROM assignments a
        JOIN users u ON u.id = a.recipient_id
        WHERE a.season_id = $1
          AND a.receipt_status = 'no_response'
          AND a.receipt_nudge_sent_at IS NULL
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let message = td_string!(Locale::uk, sms_receipt_nudge_body);

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for target in targets {
        match sms::send_sms(&config, &http_client, &target.phone, message).await {
            Ok(()) => {
                let _ = sqlx::query!(
                    r#"UPDATE assignments SET receipt_nudge_sent_at = now() WHERE id = $1"#,
                    target.id,
                )
                .execute(&pool)
                .await;
                sent += 1;
            }
            Err(e) => {
                tracing::warn!(phone = %target.phone, error = %e, "SMS send failed");
                failed_phones.push(target.phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

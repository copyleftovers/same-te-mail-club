use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

// ── Types ──────────────────────────────────────────────────────────────────────

/// Unified admin state: everything the admin UI needs in one fetch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdminState {
    pub season: Option<AdminSeason>,
    /// Total active participants (role = 'participant', status = 'active').
    pub participant_count: i64,
}

/// Full season data for the unified admin view.
///
/// Combines season metadata, enrollment counts, SMS target counts,
/// and assignment status into a single fetch.
///
/// Timestamps are pre-formatted as Ukrainian display strings by the server
/// function so SSR and WASM render identical text (no hydration mismatch).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdminSeason {
    pub id: uuid::Uuid,
    pub phase: crate::types::Phase,
    /// Pre-formatted signup deadline, e.g. "25 березня 2026, 21:09".
    pub signup_deadline: String,
    /// Pre-formatted confirm deadline, e.g. "25 березня 2026, 21:09".
    pub confirm_deadline: String,
    pub theme: Option<String>,
    pub launched: bool,
    pub enrolled_count: i64,
    pub confirmed_count: i64,
    /// Participants who actively reported they did not receive their package.
    pub not_received_count: i64,
    /// True when at least one assignment row exists for this season.
    pub assignments_released: bool,
    // SMS target counts
    pub active_user_count: i64,
    pub unnotified_sender_count: i64,
    pub unconfirmed_enrolled_count: i64,
    pub no_response_count: i64,
}

// ── SSR-only row types ─────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
struct AdminSeasonRow {
    id: uuid::Uuid,
    phase: crate::types::Phase,
    signup_deadline: time::OffsetDateTime,
    confirm_deadline: time::OffsetDateTime,
    theme: Option<String>,
    launched: bool,
    enrolled_count: i64,
    confirmed_count: i64,
    not_received_count: i64,
    assignments_released: bool,
}

// ── SSR-only helpers ───────────────────────────────────────────────────────────

/// Query active participant count.
#[cfg(feature = "ssr")]
async fn query_participant_count(pool: &sqlx::PgPool) -> Result<i64, ServerFnError> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!: i64" FROM users WHERE status = 'active' AND role = 'participant'"#,
    )
    .fetch_one(pool)
    .await
    .map_err(db_err)
}

/// Query SMS-related counts for a season.
///
/// Returns `(active_user_count, unnotified_sender_count, unconfirmed_enrolled_count, no_response_count)`.
#[cfg(feature = "ssr")]
async fn query_sms_counts(
    pool: &sqlx::PgPool,
    season_id: uuid::Uuid,
) -> Result<(i64, i64, i64, i64), ServerFnError> {
    let active_user_count = query_participant_count(pool).await?;

    let unnotified_sender_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM assignments a
        WHERE a.season_id = $1 AND a.notified_at IS NULL
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
    .map_err(db_err)?;

    let unconfirmed_enrolled_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM enrollments e
        WHERE e.season_id = $1 AND e.confirmed_ready_at IS NULL
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
    .map_err(db_err)?;

    let no_response_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM assignments a
        WHERE a.season_id = $1 AND a.receipt_status = 'no_response'
        "#,
        season_id,
    )
    .fetch_one(pool)
    .await
    .map_err(db_err)?;

    Ok((
        active_user_count,
        unnotified_sender_count,
        unconfirmed_enrolled_count,
        no_response_count,
    ))
}

// ── Server function ────────────────────────────────────────────────────────────

/// Get unified admin state (admin only).
///
/// Returns full season data (if any season exists) plus the active participant
/// count. Uses a single season query plus four scalar queries for SMS counts.
///
/// Season selection mirrors the dashboard approach: most recent season by
/// `created_at DESC LIMIT 1` with no phase filter, so complete and cancelled
/// seasons are included.
///
/// # Errors
///
/// Returns `Err` if caller is not admin or DB fails.
#[server]
pub async fn get_admin_state() -> Result<AdminState, ServerFnError> {
    use crate::{auth, types::Phase};

    let (pool, _user) = auth::require_admin().await?;

    let participant_count = query_participant_count(&pool).await?;

    let season_row = sqlx::query_as!(
        AdminSeasonRow,
        r#"
        SELECT
            s.id,
            s.phase                                               AS "phase: Phase",
            s.signup_deadline,
            s.confirm_deadline,
            s.theme,
            (s.launched_at IS NOT NULL)                           AS "launched!: bool",
            COUNT(DISTINCT e.user_id)                             AS "enrolled_count!: i64",
            COUNT(DISTINCT e.user_id) FILTER (
                WHERE e.confirmed_ready_at IS NOT NULL
            )                                                     AS "confirmed_count!: i64",
            COUNT(DISTINCT a.id) FILTER (
                WHERE a.receipt_status = 'not_received'
            )                                                     AS "not_received_count!: i64",
            (COUNT(DISTINCT a.sender_id) > 0)                     AS "assignments_released!: bool"
        FROM seasons s
        LEFT JOIN enrollments e ON e.season_id = s.id
        LEFT JOIN assignments a ON a.season_id = s.id
        GROUP BY s.id
        ORDER BY s.created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?;

    let Some(row) = season_row else {
        return Ok(AdminState {
            season: None,
            participant_count,
        });
    };

    let (active_user_count, unnotified_sender_count, unconfirmed_enrolled_count, no_response_count) =
        query_sms_counts(&pool, row.id).await?;

    Ok(AdminState {
        season: Some(AdminSeason {
            id: row.id,
            phase: row.phase,
            signup_deadline: crate::date_format::format_date_uk(row.signup_deadline),
            confirm_deadline: crate::date_format::format_date_uk(row.confirm_deadline),
            theme: row.theme,
            launched: row.launched,
            enrolled_count: row.enrolled_count,
            confirmed_count: row.confirmed_count,
            not_received_count: row.not_received_count,
            assignments_released: row.assignments_released,
            active_user_count,
            unnotified_sender_count,
            unconfirmed_enrolled_count,
            no_response_count,
        }),
        participant_count,
    })
}

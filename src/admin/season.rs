use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

// ── SSR-only row types ─────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
struct ActiveSeasonRow {
    id: uuid::Uuid,
    phase: crate::types::Phase,
}

// ── Server functions ───────────────────────────────────────────────────────────

/// Create a new season (admin only).
///
/// Inserts with `launched_at = NULL`. Season is NOT visible to participants until launched.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, deadlines are invalid, or an active season exists.
#[server]
pub async fn create_season(
    signup_deadline: String,
    confirm_deadline: String,
    theme: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth;
    use crate::i18n::i18n::{Locale, td_string};
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    let (pool, _user) = auth::require_admin().await?;

    // Parse ISO 8601 deadlines — datetime-local inputs send "YYYY-MM-DDTHH:MM"
    // which is not valid RFC3339. Append ":00Z" to make it parseable as UTC.
    let parse_deadline = |s: &str| -> Result<OffsetDateTime, ServerFnError> {
        let normalized = if s.contains('Z') || s.contains('+') {
            s.to_owned()
        } else {
            format!("{s}:00Z")
        };
        OffsetDateTime::parse(&normalized, &Rfc3339)
            .map_err(|_| ServerFnError::new(format!("invalid deadline format: {s}")))
    };

    let signup_dt = parse_deadline(&signup_deadline)?;
    let confirm_dt = parse_deadline(&confirm_deadline)?;

    let now = OffsetDateTime::now_utc();
    if signup_dt <= now {
        return Err(ServerFnError::new(td_string!(
            Locale::uk,
            season_error_signup_deadline_past
        )));
    }
    if confirm_dt <= now {
        return Err(ServerFnError::new(td_string!(
            Locale::uk,
            season_error_confirm_deadline_past
        )));
    }
    if signup_dt >= confirm_dt {
        return Err(ServerFnError::new(td_string!(
            Locale::uk,
            season_error_signup_after_confirm
        )));
    }

    let trimmed_theme = theme.and_then(|t| {
        let t = t.trim().to_owned();
        if t.is_empty() { None } else { Some(t) }
    });

    sqlx::query!(
        r#"
        INSERT INTO seasons (phase, signup_deadline, confirm_deadline, theme, launched_at)
        VALUES ('enrollment', $1, $2, $3, NULL)
        "#,
        signup_dt,
        confirm_dt,
        trimmed_theme,
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error()
            && db_err.code().as_deref() == Some("23505")
        {
            return ServerFnError::new(td_string!(Locale::uk, season_error_active_exists));
        }
        ServerFnError::new(format!("database error: {e}"))
    })?;

    Ok(())
}

/// Launch the active season, making it visible to participants.
///
/// Sets `launched_at = now()`. Only works if the season has `launched_at IS NULL`.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, no unlaunched active season, or DB fails.
#[server]
pub async fn launch_season() -> Result<(), ServerFnError> {
    use crate::auth;

    let (pool, _user) = auth::require_admin().await?;

    let rows_affected = sqlx::query!(
        r#"
        UPDATE seasons
        SET launched_at = now()
        WHERE phase NOT IN ('complete', 'cancelled')
          AND launched_at IS NULL
        "#,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(ServerFnError::new("no unlaunched active season found"));
    }

    Ok(())
}

/// Advance the active season to the next phase (admin only).
///
/// Requires the season to be launched (`launched_at IS NOT NULL`).
/// Transition logic lives in `types::Phase::try_advance`.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, no active launched season, or transition is invalid.
#[server]
pub async fn advance_season() -> Result<(), ServerFnError> {
    use crate::{auth, error::AppError, types::Phase};

    let (pool, _user) = auth::require_admin().await?;

    let season = sqlx::query_as!(
        ActiveSeasonRow,
        r#"
        SELECT id, phase AS "phase: Phase"
        FROM seasons
        WHERE phase NOT IN ('complete', 'cancelled')
          AND launched_at IS NOT NULL
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("no active launched season found"))?;

    let next_phase = season
        .phase
        .try_advance()
        .map_err(|e| AppError::from(e).into_server_fn_error())?;

    sqlx::query!(
        r#"UPDATE seasons SET phase = $1 WHERE id = $2"#,
        next_phase as Phase,
        season.id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?;

    Ok(())
}

/// Cancel the active season (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin, no active season, or season is terminal.
#[server]
pub async fn cancel_season() -> Result<(), ServerFnError> {
    use crate::{auth, error::AppError, types::Phase};

    let (pool, _user) = auth::require_admin().await?;

    let season = sqlx::query_as!(
        ActiveSeasonRow,
        r#"
        SELECT id, phase AS "phase: Phase"
        FROM seasons
        WHERE phase NOT IN ('complete', 'cancelled')
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("no active season found"))?;

    season
        .phase
        .cancel()
        .map_err(|e| AppError::from(e).into_server_fn_error())?;

    sqlx::query!(
        r#"UPDATE seasons SET phase = 'cancelled' WHERE id = $1"#,
        season.id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?;

    Ok(())
}

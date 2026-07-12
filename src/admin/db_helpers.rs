//! Shared DB types and helpers for admin server functions.
//!
//! The canonical definition of "active launched season" lives here.
//! Every server function that needs a non-terminal, launched season
//! calls `fetch_active_launched_season` instead of re-stating the predicate.

#[cfg(feature = "ssr")]
pub(super) struct ActiveSeasonRow {
    pub(super) id: uuid::Uuid,
    pub(super) phase: crate::types::Phase,
}

/// Fetch the single active launched season, if one exists.
///
/// A season is "active launched" when it is not terminal (`phase NOT IN
/// ('complete', 'cancelled')`) and has been made visible to participants
/// (`launched_at IS NOT NULL`).
///
/// Returns `None` when no such season exists (not yet launched, or all
/// seasons have reached a terminal phase).
///
/// # Errors
///
/// Returns `Err` on a database failure.
#[cfg(feature = "ssr")]
pub(super) async fn fetch_active_launched_season(
    pool: &sqlx::PgPool,
) -> Result<Option<ActiveSeasonRow>, sqlx::Error> {
    sqlx::query_as!(
        ActiveSeasonRow,
        r#"
        SELECT id, phase AS "phase: crate::types::Phase"
        FROM seasons
        WHERE phase NOT IN ('complete', 'cancelled')
          AND launched_at IS NOT NULL
        "#,
    )
    .fetch_optional(pool)
    .await
}

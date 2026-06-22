use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticipantSummary {
    pub id: uuid::Uuid,
    pub phone: String,
    pub name: String,
    pub status: crate::types::UserStatus,
    pub onboarded: bool,
}

// Row type used by list_participants query (SSR only)
#[cfg(feature = "ssr")]
struct ParticipantRow {
    id: uuid::Uuid,
    phone: String,
    name: String,
    status: crate::types::UserStatus,
    onboarded: bool,
}

// ── Server functions ──────────────────────────────────────────────────────────

/// List all participants (admin only).
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin or DB fails.
#[server]
pub async fn list_participants() -> Result<Vec<ParticipantSummary>, ServerFnError> {
    use crate::auth;

    let (pool, _user) = auth::require_admin().await?;

    let rows = sqlx::query_as!(
        ParticipantRow,
        r#"
        SELECT
            id,
            phone,
            name,
            status AS "status: crate::types::UserStatus",
            onboarded
        FROM users
        WHERE role = 'participant'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    Ok(rows
        .into_iter()
        .map(|r| ParticipantSummary {
            id: r.id,
            phone: r.phone,
            name: r.name,
            status: r.status,
            onboarded: r.onboarded,
        })
        .collect())
}

/// Deactivate a participant account (admin only).
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin or DB fails.
#[server]
pub async fn deactivate_participant(user_id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth;

    let (pool, _user) = auth::require_admin().await?;

    sqlx::query!(
        "UPDATE users SET status = 'deactivated' WHERE id = $1",
        user_id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?;

    // Revoke all active sessions immediately — deactivated user cannot continue
    // using existing browser sessions on the next request.
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;

    Ok(())
}

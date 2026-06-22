//! Admin server functions for invite code management.
//!
//! Provides CRUD operations over `invite_codes` for the admin panel:
//! - `generate_invite_code` — create a new code linked to a distributor
//! - `list_invite_codes` — read all codes with distributor and redeemer names
//! - `revoke_invite_code` — cancel an unused code
//! - `list_distributor_options` — enumerate active users eligible as distributors

#[cfg(feature = "ssr")]
use crate::error::db_err;
use crate::types::InviteCodeStatus;
use leptos::prelude::*;

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// A single invite code record returned by `list_invite_codes`.
///
/// Includes resolved names for the distributor and (optionally) the redeemer
/// so the admin UI can render the list without additional lookups.
///
/// Timestamps are pre-formatted as Ukrainian display strings by the server
/// function so SSR and WASM render identical text (no hydration mismatch).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InviteCodeRow {
    pub id: uuid::Uuid,
    pub code: String,
    pub distributor_name: String,
    pub status: InviteCodeStatus,
    pub redeemer_name: Option<String>,
    /// Pre-formatted redemption timestamp, e.g. "25 березня 2026, 21:09".
    /// `None` when the code has not been redeemed.
    pub redeemed_at: Option<String>,
}

/// A user eligible to be selected as a code distributor.
///
/// Only active users appear here; the admin is included naturally since they
/// are an active user.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistributorOption {
    pub id: uuid::Uuid,
    pub name: String,
}

// ── SSR-only query row types ──────────────────────────────────────────────────

/// Private query row for `list_invite_codes` — not exposed outside this module.
#[cfg(feature = "ssr")]
struct InviteCodeQueryRow {
    id: uuid::Uuid,
    code: String,
    distributor_name: String,
    status: InviteCodeStatus,
    redeemer_name: Option<String>,
    redeemed_at: Option<time::OffsetDateTime>,
}

/// Private query row for `list_distributor_options` — not exposed outside this module.
#[cfg(feature = "ssr")]
struct DistributorQueryRow {
    id: uuid::Uuid,
    name: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

/// Generate a new invite code linked to the given distributor (admin only).
///
/// Validates that `distributor_id` belongs to an active user, generates a
/// unique two-word code via `invite_codes::generate_unique_code`, and inserts
/// a new `invite_codes` row with status `'unused'`.
///
/// Returns the raw code string so the admin UI can display it immediately.
///
/// # Errors
///
/// Returns `Err` if:
/// - the caller is not an admin
/// - `distributor_id` does not match an active user
/// - code generation fails (20 consecutive collisions — astronomically unlikely)
/// - database write fails
#[server(GenerateInviteCode)]
pub async fn generate_invite_code(distributor_id: uuid::Uuid) -> Result<String, ServerFnError> {
    use crate::{auth, invite_codes};

    let (pool, _admin) = auth::require_admin().await?;

    // Validate that the distributor is an active user.
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM users
            WHERE id = $1 AND status = 'active'
        ) AS "exists!"
        "#,
        distributor_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    if !exists {
        return Err(ServerFnError::new("distributor must be an active user"));
    }

    let code = invite_codes::generate_unique_code(&pool)
        .await
        .map_err(db_err)?;

    sqlx::query!(
        r#"
        INSERT INTO invite_codes (code, distributor_id, status)
        VALUES ($1, $2, 'unused')
        "#,
        code,
        distributor_id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?;

    Ok(code)
}

/// List all invite codes, most recent first (admin only).
///
/// Joins with `users` twice to resolve both the distributor name and the
/// optional redeemer name in a single query.
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin or the database query fails.
#[server(ListInviteCodes)]
pub async fn list_invite_codes() -> Result<Vec<InviteCodeRow>, ServerFnError> {
    use crate::auth;

    let (pool, _admin) = auth::require_admin().await?;

    let rows = sqlx::query_as!(
        InviteCodeQueryRow,
        r#"
        SELECT
            ic.id,
            ic.code,
            d.name AS distributor_name,
            ic.status AS "status: InviteCodeStatus",
            r.name AS "redeemer_name?",
            ic.redeemed_at AS "redeemed_at?"
        FROM invite_codes ic
        JOIN users d ON d.id = ic.distributor_id
        LEFT JOIN users r ON r.id = ic.redeemer_id
        ORDER BY ic.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    Ok(rows
        .into_iter()
        .map(|r| InviteCodeRow {
            id: r.id,
            code: r.code,
            distributor_name: r.distributor_name,
            status: r.status,
            redeemer_name: r.redeemer_name,
            redeemed_at: r.redeemed_at.map(crate::date_format::format_date_uk),
        })
        .collect())
}

/// Revoke an unused invite code by its id (admin only).
///
/// Verifies the code exists and is currently `'unused'` before revoking.
/// Returns an error if the code is `'used'` (already redeemed) or already
/// `'revoked'`.
///
/// # Errors
///
/// Returns `Err` if:
/// - the caller is not an admin
/// - no code with the given id exists
/// - the code is not in `'unused'` status
/// - the database update fails
#[server(RevokeInviteCode)]
pub async fn revoke_invite_code(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth;

    let (pool, _admin) = auth::require_admin().await?;

    // Use a transaction with SELECT ... FOR UPDATE to prevent a race between
    // revoke and concurrent redemption.
    let mut tx = pool.begin().await.map_err(db_err)?;

    let status = sqlx::query_scalar!(
        r#"
        SELECT status AS "status: InviteCodeStatus"
        FROM invite_codes
        WHERE id = $1
        FOR UPDATE
        "#,
        id,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_err)?;

    match status {
        None => {
            return Err(ServerFnError::new("invite code not found"));
        }
        Some(InviteCodeStatus::Used) => {
            return Err(ServerFnError::new(
                "cannot revoke a code that has already been used",
            ));
        }
        Some(InviteCodeStatus::Revoked) => {
            return Err(ServerFnError::new("code is already revoked"));
        }
        Some(InviteCodeStatus::Unused) => {}
    }

    sqlx::query!(
        r#"
        UPDATE invite_codes
        SET status = 'revoked', revoked_at = now()
        WHERE id = $1
        "#,
        id,
    )
    .execute(&mut *tx)
    .await
    .map_err(db_err)?;

    tx.commit().await.map_err(db_err)?;

    Ok(())
}

/// List all active users eligible to be selected as a distributor (admin only).
///
/// The admin appears in this list naturally because they are an active user.
/// Results are ordered alphabetically by name for a stable dropdown.
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin or the database query fails.
#[server(ListDistributorOptions)]
pub async fn list_distributor_options() -> Result<Vec<DistributorOption>, ServerFnError> {
    use crate::auth;

    let (pool, _admin) = auth::require_admin().await?;

    let rows = sqlx::query_as!(
        DistributorQueryRow,
        r#"
        SELECT id, name
        FROM users
        WHERE status = 'active'
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    Ok(rows
        .into_iter()
        .map(|r| DistributorOption {
            id: r.id,
            name: r.name,
        })
        .collect())
}

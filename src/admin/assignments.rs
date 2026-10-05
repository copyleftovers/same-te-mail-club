use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

// ── Types (shared between SSR and WASM) ──────────────────────────────────────

/// Preview returned after generating assignments.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssignmentPreview {
    pub season_id: String,
    pub cohorts: Vec<CohortPreview>,
    pub phase: crate::types::Phase,
}

/// A single cohort's cycle visualization data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CohortPreview {
    pub score: u32,
    pub chain: Vec<AssignmentLink>,
}

/// A single sender→recipient link in the cycle.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssignmentLink {
    pub sender_id: String,
    pub sender_name: String,
    pub recipient_id: String,
    pub recipient_name: String,
}

// ── SSR-only row types ───────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
struct ConfirmedParticipantRow {
    user_id: uuid::Uuid,
}

#[cfg(feature = "ssr")]
struct GroupMembershipRow {
    user_id: uuid::Uuid,
    group_id: uuid::Uuid,
    weight: i32,
}

#[cfg(feature = "ssr")]
struct PastPairingRow {
    sender_id: uuid::Uuid,
    recipient_id: uuid::Uuid,
}

#[cfg(feature = "ssr")]
struct AssignmentRow {
    sender_id: uuid::Uuid,
    recipient_id: uuid::Uuid,
    sender_name: String,
    recipient_name: String,
}

// ── SSR-only helper functions ────────────────────────────────────────────────

/// Fetch social weights matrix for assignment generation.
///
/// Queries group memberships and past pairings from the DB, then builds
/// the weight matrix using the assignment algorithm.
#[cfg(feature = "ssr")]
async fn fetch_social_weights(
    pool: &sqlx::PgPool,
    participant_ids: &[uuid::Uuid],
) -> Result<std::collections::HashMap<(uuid::Uuid, uuid::Uuid), u32>, ServerFnError> {
    let group_memberships = sqlx::query_as!(
        GroupMembershipRow,
        r#"
        SELECT gm.user_id, gm.group_id, g.weight
        FROM known_group_members gm
        JOIN known_groups g ON g.id = gm.group_id
        WHERE gm.user_id = ANY($1)
        "#,
        participant_ids,
    )
    .fetch_all(pool)
    .await
    .map_err(db_err)?;

    let past_pairings = sqlx::query_as!(
        PastPairingRow,
        r#"
        SELECT a.sender_id, a.recipient_id
        FROM assignments a
        JOIN seasons s ON s.id = a.season_id
        WHERE s.phase IN ('complete', 'cancelled')
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(db_err)?;

    let gm_tuples: Vec<(uuid::Uuid, uuid::Uuid, u32)> = group_memberships
        .iter()
        .map(|r| (r.user_id, r.group_id, u32::try_from(r.weight).unwrap_or(0)))
        .collect();
    let pp_tuples: Vec<(uuid::Uuid, uuid::Uuid)> = past_pairings
        .iter()
        .map(|r| (r.sender_id, r.recipient_id))
        .collect();

    Ok(crate::assignment::build_weight_matrix(
        &gm_tuples, &pp_tuples,
    ))
}

/// Store assignment results in DB and build the preview response.
///
/// Queries participant names, inserts assignments, and constructs the
/// cohort preview data for the frontend.
#[cfg(feature = "ssr")]
async fn store_and_build_preview(
    pool: &sqlx::PgPool,
    season: &super::db_helpers::ActiveSeasonRow,
    result: &crate::assignment::AssignmentResult,
    participants: &[uuid::Uuid],
) -> Result<AssignmentPreview, ServerFnError> {
    let name_rows = sqlx::query!(
        r#"SELECT id, name FROM users WHERE id = ANY($1)"#,
        participants,
    )
    .fetch_all(pool)
    .await
    .map_err(db_err)?;

    let names: std::collections::HashMap<uuid::Uuid, String> =
        name_rows.into_iter().map(|r| (r.id, r.name)).collect();

    let mut cohort_previews = Vec::new();

    for cycle in &result.cohorts {
        let n = cycle.participants.len();
        let mut chain = Vec::with_capacity(n);

        for i in 0..n {
            let sender_id = cycle.participants[i];
            let recipient_id = cycle.participants[(i + 1) % n];

            sqlx::query!(
                r#"
                INSERT INTO assignments (season_id, sender_id, recipient_id)
                VALUES ($1, $2, $3)
                "#,
                season.id,
                sender_id,
                recipient_id,
            )
            .execute(pool)
            .await
            .map_err(db_err)?;

            chain.push(AssignmentLink {
                sender_id: sender_id.to_string(),
                sender_name: names.get(&sender_id).cloned().unwrap_or_default(),
                recipient_id: recipient_id.to_string(),
                recipient_name: names.get(&recipient_id).cloned().unwrap_or_default(),
            });
        }

        cohort_previews.push(CohortPreview {
            score: cycle.score,
            chain,
        });
    }

    Ok(AssignmentPreview {
        season_id: season.id.to_string(),
        cohorts: cohort_previews,
        phase: season.phase,
    })
}

// ── Server functions ─────────────────────────────────────────────────────────

/// Generate assignments for the active season (admin only).
///
/// Queries confirmed participants, builds social weight matrix, runs the
/// assignment algorithm, stores results in DB, and returns a preview.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, season not in Assignment phase,
/// no confirmed participants, or DB fails.
#[server(GenerateAssignments)]
pub async fn generate_assignments_action() -> Result<AssignmentPreview, ServerFnError> {
    use crate::{
        assignment::{self, AssignmentInput},
        auth,
        i18n::i18n::{Locale, td_string},
        types::Phase,
    };

    let (pool, _user) = auth::require_admin().await?;

    // Get active season in Assignment phase.
    let season = super::db_helpers::fetch_active_launched_season(&pool)
        .await
        .map_err(db_err)?
        .ok_or_else(|| {
            ServerFnError::new(td_string!(Locale::uk, season_error_no_launched_season))
        })?;

    if season.phase != Phase::Assignment {
        return Err(ServerFnError::new(td_string!(
            Locale::uk,
            assignments_error_wrong_phase
        )));
    }

    // Delete existing assignments for this season (idempotent — re-generate).
    sqlx::query!("DELETE FROM assignments WHERE season_id = $1", season.id)
        .execute(&pool)
        .await
        .map_err(db_err)?;

    // Get confirmed participants.
    let confirmed = sqlx::query_as!(
        ConfirmedParticipantRow,
        r#"
        SELECT user_id
        FROM enrollments
        WHERE season_id = $1 AND confirmed_ready_at IS NOT NULL
        "#,
        season.id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    if confirmed.len() < 3 {
        return Err(ServerFnError::new(td_string!(
            Locale::uk,
            assignments_error_too_few_participants
        )));
    }

    let participant_ids: Vec<uuid::Uuid> = confirmed.iter().map(|r| r.user_id).collect();

    // Build social weight matrix.
    let social_weights = fetch_social_weights(&pool, &participant_ids).await?;

    let input = AssignmentInput {
        participants: participant_ids,
        social_weights,
    };

    let result = assignment::generate_assignments(&input);
    assignment::validate_cycles(&result)
        .map_err(|e| ServerFnError::new(format!("invalid cycle topology: {e}")))?;

    // Store assignments in DB and build preview.
    store_and_build_preview(&pool, &season, &result, &input.participants).await
}

/// Exchange two participants' positions in the assignment cycle (admin only).
///
/// Every loop keeps its members and length (see `assignment::swap_positions`).
/// Runs in one transaction: lock season, read, compute, validate, rewrite, commit.
/// Any error rolls back; nothing partial is ever persisted.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, the season is not launched and in the
/// Assignment phase, both senders are the same, either has no assignment, or
/// the result fails validation.
#[server(SwapAssignment)]
pub async fn swap_assignment(
    season_id: String,
    sender_a: String,
    sender_b: String,
) -> Result<(), ServerFnError> {
    use crate::{
        assignment::{self, SwapError},
        auth,
        i18n::i18n::{Locale, td_string},
        types::Phase,
    };

    let (pool, _user) = auth::require_admin().await?;

    let sid: uuid::Uuid = season_id
        .parse()
        .map_err(|_| ServerFnError::new("invalid season_id"))?;
    let sa: uuid::Uuid = sender_a
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_a"))?;
    let sb: uuid::Uuid = sender_b
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_b"))?;

    let mut tx = pool.begin().await.map_err(db_err)?;

    // Lock the season row: serializes concurrent swaps, and makes advance_season's
    // phase UPDATE wait until this swap commits.
    let phase = sqlx::query_scalar!(
        r#"
        SELECT phase AS "phase: Phase"
        FROM seasons
        WHERE id = $1 AND launched_at IS NOT NULL
        FOR UPDATE
        "#,
        sid,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_err)?;
    if phase != Some(Phase::Assignment) {
        return Err(ServerFnError::new(td_string!(
            Locale::uk,
            assignments_error_wrong_phase
        )));
    }

    let edges: Vec<(uuid::Uuid, uuid::Uuid)> = sqlx::query!(
        r#"SELECT sender_id, recipient_id FROM assignments WHERE season_id = $1"#,
        sid,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(db_err)?
    .into_iter()
    .map(|row| (row.sender_id, row.recipient_id))
    .collect();

    let swapped = assignment::swap_positions(&edges, sa, sb).map_err(|e| {
        ServerFnError::new(match e {
            SwapError::SameParticipant => {
                td_string!(Locale::uk, assignments_error_swap_same_participant)
            }
            SwapError::NotAssigned(id) if id == sa => {
                td_string!(Locale::uk, assignments_error_sender_a_not_found)
            }
            SwapError::NotAssigned(_) => {
                td_string!(Locale::uk, assignments_error_sender_b_not_found)
            }
        })
    })?;

    // Validate BEFORE any write. Holds by construction; checked anyway.
    assignment::validated_cycles_from_edges(&swapped).map_err(|_| {
        ServerFnError::new(td_string!(Locale::uk, assignments_error_swap_breaks_cycle))
    })?;

    // Rewrite the season's rows in one statement pair: UNIQUE(season_id,
    // sender_id) and UNIQUE(season_id, recipient_id) are non-deferrable, so
    // per-row UPDATEs would collide mid-swap. Safe to recreate rows: in the
    // Assignment phase no SMS/receipt state exists yet (notified_at,
    // receipt_* stay at defaults).
    sqlx::query!("DELETE FROM assignments WHERE season_id = $1", sid)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
    let (senders, recipients): (Vec<uuid::Uuid>, Vec<uuid::Uuid>) = swapped.into_iter().unzip();
    sqlx::query!(
        r#"
        INSERT INTO assignments (season_id, sender_id, recipient_id)
        SELECT $1, s, r FROM UNNEST($2::uuid[], $3::uuid[]) AS t(s, r)
        "#,
        sid,
        &senders[..],
        &recipients[..],
    )
    .execute(&mut *tx)
    .await
    .map_err(db_err)?;

    tx.commit().await.map_err(db_err)?;
    Ok(())
}

/// Get assignment preview for the active season (admin only).
///
/// Returns the current assignments with cycle visualization data,
/// or `None` if no assignments exist yet.
///
/// # Errors
///
/// Returns `Err` if caller is not admin or DB fails.
#[server(GetAssignmentPreview)]
pub async fn get_assignment_preview() -> Result<Option<AssignmentPreview>, ServerFnError> {
    use crate::{
        auth,
        i18n::i18n::{Locale, td_string},
    };

    let (pool, _user) = auth::require_admin().await?;

    let season = super::db_helpers::fetch_active_launched_season(&pool)
        .await
        .map_err(db_err)?;

    let Some(season) = season else {
        return Ok(None);
    };

    let assignments = sqlx::query_as!(
        AssignmentRow,
        r#"
        SELECT
            a.sender_id,
            a.recipient_id,
            su.name AS sender_name,
            ru.name AS recipient_name
        FROM assignments a
        JOIN users su ON su.id = a.sender_id
        JOIN users ru ON ru.id = a.recipient_id
        WHERE a.season_id = $1
        ORDER BY a.created_at, a.sender_id
        "#,
        season.id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    if assignments.is_empty() {
        // Get confirmed count for the "before generate" view.
        return Ok(Some(AssignmentPreview {
            season_id: season.id.to_string(),
            cohorts: Vec::new(),
            phase: season.phase,
        }));
    }

    let by_sender: std::collections::HashMap<uuid::Uuid, &AssignmentRow> =
        assignments.iter().map(|a| (a.sender_id, a)).collect();
    let edges: Vec<(uuid::Uuid, uuid::Uuid)> = assignments
        .iter()
        .map(|a| (a.sender_id, a.recipient_id))
        .collect();
    let result = crate::assignment::cycles_from_edges(&edges).map_err(|_| {
        ServerFnError::new(td_string!(Locale::uk, season_error_assignments_invalid))
    })?;

    let cohorts = result
        .cohorts
        .iter()
        .map(|cycle| CohortPreview {
            score: cycle.score,
            chain: cycle
                .participants
                .iter()
                .filter_map(|id| by_sender.get(id))
                .map(|a| AssignmentLink {
                    sender_id: a.sender_id.to_string(),
                    sender_name: a.sender_name.clone(),
                    recipient_id: a.recipient_id.to_string(),
                    recipient_name: a.recipient_name.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(Some(AssignmentPreview {
        season_id: season.id.to_string(),
        cohorts,
        phase: season.phase,
    }))
}

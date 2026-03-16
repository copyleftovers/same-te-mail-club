use leptos::prelude::*;

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
    pub sender_name: String,
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

#[cfg(feature = "ssr")]
struct ActiveSeasonRow {
    id: uuid::Uuid,
    phase: crate::types::Phase,
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
// The query-heavy nature of this function makes splitting it less readable.
#[allow(clippy::too_many_lines)]
#[server(GenerateAssignments)]
pub async fn generate_assignments_action() -> Result<AssignmentPreview, ServerFnError> {
    use crate::{
        assignment::{self, AssignmentInput},
        auth,
        error::AppError,
        types::{Phase, UserRole},
    };
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

    // Get active season in Assignment phase.
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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("no active launched season found"))?;

    if season.phase != Phase::Assignment {
        return Err(ServerFnError::new(
            "season must be in assignment phase to generate assignments",
        ));
    }

    // Delete existing assignments for this season (idempotent — re-generate).
    sqlx::query!("DELETE FROM assignments WHERE season_id = $1", season.id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if confirmed.len() < 3 {
        return Err(ServerFnError::new(
            "need at least 3 confirmed participants to generate assignments",
        ));
    }

    let participant_ids: Vec<uuid::Uuid> = confirmed.iter().map(|r| r.user_id).collect();

    // Build social weight matrix.
    let group_memberships = sqlx::query_as!(
        GroupMembershipRow,
        r#"
        SELECT gm.user_id, gm.group_id, g.weight
        FROM known_group_members gm
        JOIN known_groups g ON g.id = gm.group_id
        WHERE gm.user_id = ANY($1)
        "#,
        &participant_ids,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    let past_pairings = sqlx::query_as!(
        PastPairingRow,
        r#"
        SELECT a.sender_id, a.recipient_id
        FROM assignments a
        JOIN seasons s ON s.id = a.season_id
        WHERE s.phase IN ('complete', 'cancelled')
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    let gm_tuples: Vec<(uuid::Uuid, uuid::Uuid, u32)> = group_memberships
        .iter()
        .map(|r| (r.user_id, r.group_id, u32::try_from(r.weight).unwrap_or(0)))
        .collect();
    let pp_tuples: Vec<(uuid::Uuid, uuid::Uuid)> = past_pairings
        .iter()
        .map(|r| (r.sender_id, r.recipient_id))
        .collect();

    let social_weights = assignment::build_weight_matrix(&gm_tuples, &pp_tuples);

    let input = AssignmentInput {
        participants: participant_ids,
        social_weights,
    };

    let result = assignment::generate_assignments(&input);
    assignment::validate_cycles(&result)
        .map_err(|e| ServerFnError::new(format!("invalid cycle topology: {e}")))?;

    // Store assignments in DB and build preview.
    // First, build a name lookup.
    let name_rows = sqlx::query!(
        r#"SELECT id, name FROM users WHERE id = ANY($1)"#,
        &input.participants,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

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
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

            chain.push(AssignmentLink {
                sender_name: names.get(&sender_id).cloned().unwrap_or_default(),
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

/// Swap two senders' recipients (admin only).
///
/// A's recipient becomes B's, and B's becomes A's.
/// Validates the resulting topology still forms valid cycles.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, assignments don't exist,
/// or the swap breaks cycle topology.
// Swap involves loading, updating, and validating — splitting would scatter the transaction.
#[allow(clippy::too_many_lines)]
#[server(SwapAssignment)]
pub async fn swap_assignment(
    season_id: String,
    sender_a: String,
    sender_b: String,
) -> Result<(), ServerFnError> {
    use crate::{auth, error::AppError, types::UserRole};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

    let sid: uuid::Uuid = season_id
        .parse()
        .map_err(|_| ServerFnError::new("invalid season_id"))?;
    let sa: uuid::Uuid = sender_a
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_a"))?;
    let sb: uuid::Uuid = sender_b
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_b"))?;

    // Load both assignments.
    let rec_a = sqlx::query_scalar!(
        r#"SELECT recipient_id FROM assignments WHERE season_id = $1 AND sender_id = $2"#,
        sid,
        sa,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("assignment for sender_a not found"))?;

    let rec_b = sqlx::query_scalar!(
        r#"SELECT recipient_id FROM assignments WHERE season_id = $1 AND sender_id = $2"#,
        sid,
        sb,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("assignment for sender_b not found"))?;

    // Perform swap.
    sqlx::query!(
        r#"UPDATE assignments SET recipient_id = $1 WHERE season_id = $2 AND sender_id = $3"#,
        rec_b,
        sid,
        sa,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    sqlx::query!(
        r#"UPDATE assignments SET recipient_id = $1 WHERE season_id = $2 AND sender_id = $3"#,
        rec_a,
        sid,
        sb,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    // Validate resulting topology.
    let all_assignments = sqlx::query_as!(
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
        "#,
        sid,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    // Build cycle representation for validation.
    let cycle_participants: Vec<uuid::Uuid> = all_assignments.iter().map(|a| a.sender_id).collect();
    let mut ordered = Vec::new();
    let next_map: std::collections::HashMap<uuid::Uuid, uuid::Uuid> = all_assignments
        .iter()
        .map(|a| (a.sender_id, a.recipient_id))
        .collect();

    if let Some(&first) = cycle_participants.first() {
        let mut current = first;
        for _ in 0..cycle_participants.len() {
            ordered.push(current);
            if let Some(&next) = next_map.get(&current) {
                current = next;
            } else {
                return Err(ServerFnError::new("broken cycle after swap"));
            }
        }
        // Verify it's a complete cycle: last element's next should be first.
        if next_map.get(&current) != Some(&first) && current != first {
            return Err(ServerFnError::new("swap breaks cycle topology"));
        }
    }

    let validation_result = crate::assignment::AssignmentResult {
        cohorts: vec![crate::assignment::Cycle {
            participants: ordered,
            score: 0,
        }],
    };

    crate::assignment::validate_cycles(&validation_result)
        .map_err(|e| ServerFnError::new(format!("swap breaks cycle topology: {e}")))?;

    Ok(())
}

/// Release assignments by advancing the season from Assignment to Delivery (admin only).
///
/// Release IS the phase transition. There is no per-assignment released flag.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, no active season in Assignment phase,
/// or phase transition fails.
#[server(ReleaseAssignments)]
pub async fn release_assignments() -> Result<(), ServerFnError> {
    use crate::{
        auth,
        error::AppError,
        types::{Phase, UserRole},
    };
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("no active launched season found"))?;

    if season.phase != Phase::Assignment {
        return Err(ServerFnError::new(
            "season must be in assignment phase to release assignments",
        ));
    }

    // Verify assignments exist.
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!: i64" FROM assignments WHERE season_id = $1"#,
        season.id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if count == 0 {
        return Err(ServerFnError::new(
            "no assignments to release — generate assignments first",
        ));
    }

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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

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
        error::AppError,
        types::{Phase, UserRole},
    };
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

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
        ORDER BY a.created_at
        "#,
        season.id,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if assignments.is_empty() {
        // Get confirmed count for the "before generate" view.
        return Ok(Some(AssignmentPreview {
            season_id: season.id.to_string(),
            cohorts: Vec::new(),
            phase: season.phase,
        }));
    }

    // Build chain from next_map (sender → recipient).
    let next_map: std::collections::HashMap<uuid::Uuid, &AssignmentRow> =
        assignments.iter().map(|a| (a.sender_id, a)).collect();

    // Walk the cycle starting from first sender.
    let mut chain = Vec::new();
    let first = assignments[0].sender_id;
    let mut current = first;
    for _ in 0..assignments.len() {
        if let Some(a) = next_map.get(&current) {
            chain.push(AssignmentLink {
                sender_name: a.sender_name.clone(),
                recipient_name: a.recipient_name.clone(),
            });
            current = a.recipient_id;
        } else {
            break;
        }
    }

    Ok(Some(AssignmentPreview {
        season_id: season.id.to_string(),
        cohorts: vec![CohortPreview { score: 0, chain }],
        phase: season.phase,
    }))
}

/// Get confirmed count for the active season (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin or DB fails.
#[server(GetConfirmedCount)]
pub async fn get_confirmed_count() -> Result<i64, ServerFnError> {
    use crate::{auth, error::AppError, types::UserRole};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM enrollments e
        JOIN seasons s ON s.id = e.season_id
        WHERE s.phase NOT IN ('complete', 'cancelled')
          AND s.launched_at IS NOT NULL
          AND e.confirmed_ready_at IS NOT NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(count)
}

// ── Component ────────────────────────────────────────────────────────────────

/// Admin assignments page (Epic 3: Stories 3.1, 3.2, 3.3).
///
/// Shows confirmed count, generate button, cycle visualization, swap UI,
/// and release button.
#[component]
pub fn AssignmentsPage() -> impl IntoView {
    let generate_action = ServerAction::<GenerateAssignments>::new();
    let swap_action = ServerAction::<SwapAssignment>::new();
    let release_action = ServerAction::<ReleaseAssignments>::new();

    // Load confirmed count.
    let confirmed_count = Resource::new(|| (), |()| get_confirmed_count());

    // Load assignment preview, refetch after any mutation.
    let preview = Resource::new(
        move || {
            (
                generate_action.version().get(),
                swap_action.version().get(),
                release_action.version().get(),
            )
        },
        |_| get_assignment_preview(),
    );

    // Hydration gate.
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

    view! {
        <div class="admin-assignments">
            <h1>"Розподіл / Assignments"</h1>

            // Error display.
            {move || {
                let err = generate_action.value().get().and_then(Result::err)
                    .or_else(|| swap_action.value().get().and_then(Result::err))
                    .or_else(|| release_action.value().get().and_then(Result::err));
                err.map(|e| view! {
                    <p class="error">{e.to_string()}</p>
                })
            }}

            // Confirmed count.
            <Suspense fallback=|| view! { <p>"Завантаження..."</p> }>
                {move || confirmed_count.get().map(|result| match result {
                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                    Ok(count) => view! {
                        <p>"Підтверджено / Confirmed: " {count}</p>
                    }.into_any(),
                })}
            </Suspense>

            // Main content: preview + actions.
            <Suspense fallback=|| view! { <p>"Завантаження..."</p> }>
                {move || preview.get().map(|result| match result {
                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                    Ok(None) => view! {
                        <p>"Немає активного сезону / No active season"</p>
                    }.into_any(),
                    Ok(Some(ref p)) => {
                        render_preview(p, generate_action, swap_action, release_action, hydrated)
                    }
                })}
            </Suspense>
        </div>
    }
}

/// Render the assignment preview with generate/swap/release controls.
///
/// Extracted from `AssignmentsPage` to stay within the line limit.
fn render_preview(
    p: &AssignmentPreview,
    generate_action: ServerAction<GenerateAssignments>,
    swap_action: ServerAction<SwapAssignment>,
    release_action: ServerAction<ReleaseAssignments>,
    hydrated: ReadSignal<bool>,
) -> AnyView {
    let is_assignment_phase = p.phase == crate::types::Phase::Assignment;
    let has_assignments = !p.cohorts.is_empty() && !p.cohorts.iter().all(|c| c.chain.is_empty());
    let season_id = p.season_id.clone();

    view! {
        <div>
            // Generate or Released status.
            {if is_assignment_phase {
                view! {
                    <leptos::form::ActionForm action=generate_action>
                        <button
                            type="submit"
                            data-testid="generate-button"
                            disabled=move || !hydrated.get()
                        >
                            {if has_assignments {
                                "Перегенерувати / Regenerate"
                            } else {
                                "Згенерувати / Generate"
                            }}
                        </button>
                    </leptos::form::ActionForm>
                }.into_any()
            } else {
                view! {
                    <p>"Опубліковано / Released — assignments are visible to participants."</p>
                }.into_any()
            }}

            // Cycle visualization.
            {if has_assignments {
                render_cycle_visualization(&p.cohorts).into_any()
            } else {
                ().into_any()
            }}

            // Swap UI — only before release and when assignments exist.
            {if has_assignments && is_assignment_phase {
                let sid = season_id.clone();
                view! {
                    <SwapForm
                        swap_action=swap_action
                        season_id=sid
                        hydrated=hydrated
                    />
                }.into_any()
            } else {
                ().into_any()
            }}

            // Release button — only when assignments exist and not yet released.
            {if has_assignments && is_assignment_phase {
                view! {
                    <leptos::form::ActionForm action=release_action>
                        <button
                            type="submit"
                            data-testid="release-button"
                            disabled=move || !hydrated.get()
                        >
                            "Опублікувати / Release"
                        </button>
                    </leptos::form::ActionForm>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
    .into_any()
}

/// Render the cycle visualization section.
fn render_cycle_visualization(cohorts: &[CohortPreview]) -> impl IntoView {
    let cohorts_view = cohorts
        .iter()
        .enumerate()
        .map(|(idx, cohort)| {
            let chain_items = cohort
                .chain
                .iter()
                .map(|link| {
                    view! {
                        <li>
                            {link.sender_name.clone()}
                            " → "
                            {link.recipient_name.clone()}
                        </li>
                    }
                })
                .collect_view();

            view! {
                <div class="cohort">
                    <h3>{format!("Когорта / Cohort {}", idx + 1)}</h3>
                    <p>"Score: " {cohort.score}</p>
                    <ol>{chain_items}</ol>
                </div>
            }
        })
        .collect_view();

    view! {
        <div data-testid="cycle-visualization">
            <h2>"Цикли / Cycles"</h2>
            {cohorts_view}
        </div>
    }
}

/// Swap form sub-component.
#[component]
fn SwapForm(
    swap_action: ServerAction<SwapAssignment>,
    season_id: String,
    hydrated: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <section>
            <h3>"Swap / Обмін"</h3>
            <p>"Swap two senders' recipients. Enter their names or IDs."</p>
            <leptos::form::ActionForm action=swap_action>
                <input type="hidden" name="season_id" value=season_id />
                <div>
                    <label for="sender-a">"Sender A (UUID)"</label>
                    <input id="sender-a" type="text" name="sender_a" required=true />
                </div>
                <div>
                    <label for="sender-b">"Sender B (UUID)"</label>
                    <input id="sender-b" type="text" name="sender_b" required=true />
                </div>
                <button
                    type="submit"
                    data-testid="swap-button"
                    disabled=move || !hydrated.get()
                >
                    "Обміняти / Swap"
                </button>
            </leptos::form::ActionForm>
        </section>
    }
}

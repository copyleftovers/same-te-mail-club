use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Season info returned to admin for display and management.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeasonStatus {
    pub id: uuid::Uuid,
    pub phase: crate::types::Phase,
    pub signup_deadline: String,
    pub confirm_deadline: String,
    pub theme: Option<String>,
    pub launched: bool,
    pub enrolled_count: i64,
    pub confirmed_count: i64,
}

// ── SSR-only row types ─────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
struct ActiveSeasonRow {
    id: uuid::Uuid,
    phase: crate::types::Phase,
}

#[cfg(feature = "ssr")]
struct SeasonStatusRow {
    id: uuid::Uuid,
    phase: crate::types::Phase,
    signup_deadline: time::OffsetDateTime,
    confirm_deadline: time::OffsetDateTime,
    theme: Option<String>,
    launched: bool,
    enrolled_count: i64,
    confirmed_count: i64,
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
    use crate::{auth, error::AppError, types::UserRole};
    use http::request::Parts;
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

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
        return Err(ServerFnError::new("signup deadline must be in the future"));
    }
    if confirm_dt <= now {
        return Err(ServerFnError::new("confirm deadline must be in the future"));
    }
    if signup_dt >= confirm_dt {
        return Err(ServerFnError::new(
            "signup deadline must be before confirm deadline",
        ));
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
            return ServerFnError::new("an active season already exists");
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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
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

/// Cancel the active season (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin, no active season, or season is terminal.
#[server]
pub async fn cancel_season() -> Result<(), ServerFnError> {
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
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(())
}

/// Get the current active season status (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin or DB fails.
#[server]
pub async fn get_season_status() -> Result<Option<SeasonStatus>, ServerFnError> {
    use crate::{
        auth,
        error::AppError,
        types::{Phase, UserRole},
    };
    use http::request::Parts;
    use time::format_description::well_known::Rfc3339;

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

    let row = sqlx::query_as!(
        SeasonStatusRow,
        r#"
        SELECT
            s.id,
            s.phase AS "phase: Phase",
            s.signup_deadline,
            s.confirm_deadline,
            s.theme,
            (s.launched_at IS NOT NULL) AS "launched!: bool",
            COUNT(e.user_id) AS "enrolled_count!: i64",
            COUNT(e.confirmed_ready_at) AS "confirmed_count!: i64"
        FROM seasons s
        LEFT JOIN enrollments e ON e.season_id = s.id
        WHERE s.phase NOT IN ('complete', 'cancelled')
        GROUP BY s.id
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(row.map(|r| SeasonStatus {
        id: r.id,
        phase: r.phase,
        signup_deadline: r.signup_deadline.format(&Rfc3339).unwrap_or_default(),
        confirm_deadline: r.confirm_deadline.format(&Rfc3339).unwrap_or_default(),
        theme: r.theme,
        launched: r.launched,
        enrolled_count: r.enrolled_count,
        confirmed_count: r.confirmed_count,
    }))
}

// ── Component sub-parts ────────────────────────────────────────────────────────

#[component]
fn CreateSeasonForm(
    create_action: ServerAction<CreateSeason>,
    hydrated: ReadSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <section>
            <h2>{t!(i18n, season_create_form_title)}</h2>
            <leptos::form::ActionForm action=create_action>
                <div>
                    <label for="signup-deadline">
                        {t!(i18n, season_signup_deadline_label)}
                    </label>
                    <input
                        id="signup-deadline"
                        type="datetime-local"
                        name="signup_deadline"
                        required=true
                        data-testid="signup-deadline-input"
                    />
                </div>
                <div>
                    <label for="confirm-deadline">
                        {t!(i18n, season_confirm_deadline_label)}
                    </label>
                    <input
                        id="confirm-deadline"
                        type="datetime-local"
                        name="confirm_deadline"
                        required=true
                        data-testid="confirm-deadline-input"
                    />
                </div>
                <div>
                    <label for="theme">
                        {t!(i18n, season_theme_label)}
                    </label>
                    <input
                        id="theme"
                        type="text"
                        name="theme"
                        placeholder=move || t_string!(i18n, season_theme_placeholder)
                        data-testid="theme-input"
                    />
                </div>
                <button
                    type="submit"
                    data-testid="create-season-button"
                    disabled=move || !hydrated.get()
                >
                    {t!(i18n, season_create_button)}
                </button>
            </leptos::form::ActionForm>
        </section>
    }
}

#[component]
fn ActiveSeasonPanel(
    status: SeasonStatus,
    launch_action: ServerAction<LaunchSeason>,
    advance_action: ServerAction<AdvanceSeason>,
    cancel_action: ServerAction<CancelSeason>,
    hydrated: ReadSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let can_advance = status.phase.can_advance();
    let launched = status.launched;
    let phase_label = if launched {
        format!("{}", status.phase)
    } else {
        "created / створено".to_owned()
    };

    view! {
        <section>
            <h2>{t!(i18n, season_current_section_title)}</h2>
            <dl>
                <dt>{t!(i18n, season_phase_label)}</dt>
                <dd>{phase_label}</dd>

                {status.theme.as_ref().map(|theme_val| view! {
                    <dt>{t!(i18n, season_theme_display_label)}</dt>
                    <dd data-testid="season-theme">{theme_val.clone()}</dd>
                })}

                <dt>{t!(i18n, season_signup_deadline_display)}</dt>
                <dd data-testid="season-deadline">{status.signup_deadline.clone()}</dd>

                <dt>{t!(i18n, season_confirm_deadline_display)}</dt>
                <dd>{status.confirm_deadline.clone()}</dd>

                <dt>{t!(i18n, season_enrolled_label)}</dt>
                <dd>{status.enrolled_count.to_string()}</dd>

                <dt>{t!(i18n, season_confirmed_label)}</dt>
                <dd data-testid="confirmed-count">{status.confirmed_count.to_string()}</dd>
            </dl>

            <div class="season-actions">
                // Launch button — only shown when not yet launched
                {if launched {
                    ().into_any()
                } else {
                    view! {
                        <leptos::form::ActionForm action=launch_action>
                            <button
                                type="submit"
                                data-testid="launch-button"
                                disabled=move || !hydrated.get()
                            >
                                {t!(i18n, season_launch_button)}
                            </button>
                        </leptos::form::ActionForm>
                    }.into_any()
                }}

                // Advance button — only when launched and phase can advance
                {if launched && can_advance {
                    view! {
                        <leptos::form::ActionForm action=advance_action>
                            <button
                                type="submit"
                                data-testid="advance-button"
                                disabled=move || !hydrated.get()
                            >
                                {t!(i18n, season_advance_button)}
                            </button>
                        </leptos::form::ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}

                // Cancel button — available while launched
                {if launched {
                    view! {
                        <leptos::form::ActionForm action=cancel_action>
                            <button
                                type="submit"
                                data-testid="cancel-button"
                                disabled=move || !hydrated.get()
                            >
                                {t!(i18n, season_cancel_button)}
                            </button>
                        </leptos::form::ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
            </div>
        </section>
    }
}

// ── Page component ─────────────────────────────────────────────────────────────

/// Admin season management page (Stories 4.1, 4.2).
///
/// Shows create form when no active season exists,
/// and control buttons (launch, advance, cancel) for existing seasons.
#[component]
pub fn SeasonManagePage() -> impl IntoView {
    let i18n = use_i18n();
    let create_action = ServerAction::<CreateSeason>::new();
    let launch_action = ServerAction::<LaunchSeason>::new();
    let advance_action = ServerAction::<AdvanceSeason>::new();
    let cancel_action = ServerAction::<CancelSeason>::new();

    // Refetch status on any action completion
    let status = Resource::new(
        move || {
            (
                create_action.version().get(),
                launch_action.version().get(),
                advance_action.version().get(),
                cancel_action.version().get(),
            )
        },
        |_| get_season_status(),
    );

    // Hydration gate — prevent native POST before WASM hydrates
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

    view! {
        <div class="admin-season">
            <h1>{t!(i18n, season_page_title)}</h1>

            // Error display for any action
            {move || {
                let err = create_action.value().get().and_then(Result::err)
                    .or_else(|| launch_action.value().get().and_then(Result::err))
                    .or_else(|| advance_action.value().get().and_then(Result::err))
                    .or_else(|| cancel_action.value().get().and_then(Result::err));
                err.map(|e| view! {
                    <p class="error">{e.to_string()}</p>
                })
            }}

            <Suspense fallback=move || view! { <p>{t!(i18n, common_loading)}</p> }>
                {move || status.get().map(|result| match result {
                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                    Ok(None) => {
                        view! {
                            <CreateSeasonForm
                                create_action=create_action
                                hydrated=hydrated
                            />
                        }.into_any()
                    }
                    Ok(Some(s)) => {
                        view! {
                            <ActiveSeasonPanel
                                status=s
                                launch_action=launch_action
                                advance_action=advance_action
                                cancel_action=cancel_action
                                hydrated=hydrated
                            />
                        }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}

use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

// ── Types ─────────────────────────────────────────────────────────────────────

/// State returned to the admin dashboard.
///
/// Encodes the full picture: active season health + key counts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardState {
    pub season: Option<DashboardSeason>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardSeason {
    pub phase: crate::types::Phase,
    pub theme: Option<String>,
    pub launched: bool,
    pub enrolled_count: i64,
    pub confirmed_count: i64,
    pub not_received_count: i64,
}

// ── SSR-only row type ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
struct DashRow {
    phase: crate::types::Phase,
    theme: Option<String>,
    launched: bool,
    enrolled_count: i64,
    confirmed_count: i64,
    not_received_count: i64,
}

// ── Server function ────────────────────────────────────────────────────────────

/// Get dashboard state (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin or DB fails.
#[server]
pub async fn get_dashboard() -> Result<DashboardState, ServerFnError> {
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

    let row = sqlx::query_as!(
        DashRow,
        r#"
        SELECT
            s.phase AS "phase: Phase",
            s.theme,
            (s.launched_at IS NOT NULL) AS "launched!: bool",
            COUNT(DISTINCT e.user_id) AS "enrolled_count!: i64",
            COUNT(DISTINCT e.user_id) FILTER (WHERE e.confirmed_ready_at IS NOT NULL)
                AS "confirmed_count!: i64",
            COUNT(DISTINCT a.id) FILTER (WHERE a.receipt_status = 'not_received')
                AS "not_received_count!: i64"
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
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(DashboardState {
        season: row.map(|r| DashboardSeason {
            phase: r.phase,
            theme: r.theme,
            launched: r.launched,
            enrolled_count: r.enrolled_count,
            confirmed_count: r.confirmed_count,
            not_received_count: r.not_received_count,
        }),
    })
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Admin dashboard page.
///
/// Shows season health at a glance: phase, enrolled/confirmed counts, alerts.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let i18n = use_i18n();
    let dashboard = Resource::new(|| (), |()| get_dashboard());

    view! {
        <div class="prose-page">
            <h1>{t!(i18n, admin_dashboard_title)}</h1>

            <Suspense fallback=move || view! { <p>{t!(i18n, common_loading)}</p> }>
                {move || dashboard.get().map(|result| match result {
                    Err(e) => view! { <p class="alert">{e.to_string()}</p> }.into_any(),
                    Ok(state) => match state.season {
                        None => view! {
                            <p>{t!(i18n, dashboard_no_season)}</p>
                            <p>
                                <a class="btn" data-size="sm" href="/admin/season">{t!(i18n, dashboard_create_season_button)}</a>
                            </p>
                        }.into_any(),
                        Some(s) => {
                            let is_terminal = matches!(
                                s.phase,
                                crate::types::Phase::Complete | crate::types::Phase::Cancelled
                            );

                            let (phase_display, phase_status) = if s.launched {
                                match s.phase {
                                    crate::types::Phase::Enrollment =>
                                        (t_string!(i18n, season_phase_enrollment), "active"),
                                    crate::types::Phase::Preparation =>
                                        (t_string!(i18n, season_phase_preparation), "active"),
                                    crate::types::Phase::Assignment =>
                                        (t_string!(i18n, season_phase_assignment), "pending"),
                                    crate::types::Phase::Delivery =>
                                        (t_string!(i18n, season_phase_delivery), "active"),
                                    crate::types::Phase::Complete =>
                                        (t_string!(i18n, season_phase_complete), "confirmed"),
                                    crate::types::Phase::Cancelled =>
                                        (t_string!(i18n, season_phase_cancelled), "inactive"),
                                }
                            } else {
                                (t_string!(i18n, season_phase_created), "pending")
                            };

                            view! {
                                <section>
                                    <dl>
                                        <dt>{t!(i18n, dashboard_phase_label)}</dt>
                                        <dd><span class="badge" data-status=phase_status>{phase_display}</span></dd>

                                        {s.theme.as_ref().map(|theme_val| view! {
                                            <dt>{t!(i18n, dashboard_theme_label)}</dt>
                                            <dd>{theme_val.clone()}</dd>
                                        })}

                                        {if is_terminal {
                                            ().into_any()
                                        } else {
                                            view! {
                                                <dt>{t!(i18n, dashboard_enrolled_label)}</dt>
                                                <dd>{s.enrolled_count}</dd>

                                                <dt>{t!(i18n, dashboard_confirmed_label)}</dt>
                                                <dd>{s.confirmed_count}</dd>
                                            }.into_any()
                                        }}
                                    </dl>

                                    {if s.not_received_count > 0 {
                                        view! {
                                            <div class="alert">
                                                <strong>
                                                    {t!(i18n, dashboard_not_received_label)}
                                                    {s.not_received_count}
                                                </strong>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }}

                                    {if is_terminal {
                                        view! {
                                            <p>
                                                <a class="btn" data-size="sm" href="/admin/season">{t!(i18n, dashboard_create_season_button)}</a>
                                            </p>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }}
                                </section>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

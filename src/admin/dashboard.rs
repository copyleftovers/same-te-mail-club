use crate::components::stepper::PhaseStepper;
use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

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
    use crate::{auth, types::Phase};

    let (pool, _user) = auth::require_admin().await?;

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
    .map_err(db_err)?;

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

            <Suspense fallback=move || {
                view! {
                    <div aria-hidden="true" class="flex flex-col gap-3">
                        <div class="skeleton-line h-4 w-3/4"></div>
                        <div class="skeleton-line h-4 w-1/2"></div>
                        <div class="skeleton-line h-4 w-5/8"></div>
                    </div>
                }
            }>
                {move || {
                    dashboard
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="alert">{e.to_string()}</p> }.into_any(),
                            Ok(state) => {
                                match state.season {
                                    None => {
                                        view! {
                                            <p>{t!(i18n, dashboard_no_season)}</p>
                                            <p>
                                                <a class="btn" data-size="sm" href="/admin/season">
                                                    {t!(i18n, dashboard_create_season_button)}
                                                </a>
                                            </p>
                                        }
                                            .into_any()
                                    }
                                    Some(ref s) => render_season_detail(s, i18n),
                                }
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Render season detail section.
fn render_season_detail(
    s: &DashboardSeason,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    let is_terminal = matches!(
        s.phase,
        crate::types::Phase::Complete | crate::types::Phase::Cancelled
    );
    let (phase_display, phase_status) = if s.launched {
        match s.phase {
            crate::types::Phase::Enrollment => (t_string!(i18n, season_phase_enrollment), "active"),
            crate::types::Phase::Preparation => {
                (t_string!(i18n, season_phase_preparation), "active")
            }
            crate::types::Phase::Assignment => {
                (t_string!(i18n, season_phase_assignment), "pending")
            }
            crate::types::Phase::Delivery => (t_string!(i18n, season_phase_delivery), "active"),
            crate::types::Phase::Complete => (t_string!(i18n, season_phase_complete), "confirmed"),
            crate::types::Phase::Cancelled => (t_string!(i18n, season_phase_cancelled), "inactive"),
        }
    } else {
        (t_string!(i18n, season_phase_created), "pending")
    };

    view! {
        <section>
            <PhaseStepper current_phase=s.phase />

            <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 mb-6">
                <div class="stat-card" data-testid="stat-phase">
                    <div class="stat-label">{t!(i18n, dashboard_phase_label)}</div>
                    <div class="stat-value text-base">
                        <span class="badge" data-status=phase_status>
                            {phase_display}
                        </span>
                    </div>
                </div>

                {s
                    .theme
                    .as_ref()
                    .map(|theme_val| {
                        view! {
                            <div class="stat-card" data-testid="stat-theme">
                                <div class="stat-label">{t!(i18n, dashboard_theme_label)}</div>
                                <div class="stat-value text-lg">{theme_val.clone()}</div>
                            </div>
                        }
                    })}

                {if is_terminal {
                    ().into_any()
                } else {
                    view! {
                        <div class="stat-card" data-testid="stat-enrolled">
                            <div class="stat-label">{t!(i18n, dashboard_enrolled_label)}</div>
                            <div class="stat-value">{s.enrolled_count.to_string()}</div>
                        </div>

                        <div class="stat-card" data-testid="stat-confirmed">
                            <div class="stat-label">{t!(i18n, dashboard_confirmed_label)}</div>
                            <div class="stat-value">{s.confirmed_count.to_string()}</div>
                        </div>
                    }
                        .into_any()
                }}
            </div>

            {if s.not_received_count > 0 {
                view! {
                    <div class="alert">
                        <strong>
                            {t!(i18n, dashboard_not_received_label)}
                            {s.not_received_count}
                        </strong>
                    </div>
                }
                    .into_any()
            } else {
                ().into_any()
            }}

            {if is_terminal {
                view! {
                    <p>
                        <a class="btn" data-size="sm" href="/admin/season">
                            {t!(i18n, dashboard_create_season_button)}
                        </a>
                    </p>
                }
                    .into_any()
            } else {
                ().into_any()
            }}
        </section>
    }
    .into_any()
}

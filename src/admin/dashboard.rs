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
        WHERE s.phase NOT IN ('complete', 'cancelled')
        GROUP BY s.id
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
    let dashboard = Resource::new(|| (), |()| get_dashboard());

    view! {
        <div class="admin-dashboard">
            <h1>"Dashboard"</h1>

            <Suspense fallback=|| view! { <p>"Завантаження..."</p> }>
                {move || dashboard.get().map(|result| match result {
                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                    Ok(state) => match state.season {
                        None => view! {
                            <p>"Немає активного сезону. / No active season."</p>
                            <p>
                                <a href="/admin/season">"Створити сезон / Create season"</a>
                            </p>
                        }.into_any(),
                        Some(s) => {
                            let phase_display = if s.launched {
                                match s.phase {
                                    crate::types::Phase::Enrollment =>
                                        "signup / реєстрація".to_owned(),
                                    crate::types::Phase::Preparation =>
                                        "preparation / підготовка".to_owned(),
                                    crate::types::Phase::Assignment =>
                                        "assignment / розподіл".to_owned(),
                                    crate::types::Phase::Delivery =>
                                        "delivery / відправлення".to_owned(),
                                    crate::types::Phase::Complete =>
                                        "complete / завершено".to_owned(),
                                    crate::types::Phase::Cancelled =>
                                        "cancelled / скасовано".to_owned(),
                                }
                            } else {
                                "created / створено".to_owned()
                            };

                            view! {
                                <section>
                                    <dl>
                                        <dt>"Фаза / Phase"</dt>
                                        <dd>{phase_display}</dd>

                                        {s.theme.as_ref().map(|t| view! {
                                            <dt>"Тема"</dt>
                                            <dd>{t.clone()}</dd>
                                        })}

                                        <dt>"Зареєстровано / Enrolled"</dt>
                                        <dd>{s.enrolled_count}</dd>

                                        <dt>"Підтверджено / Confirmed"</dt>
                                        <dd>{s.confirmed_count}</dd>
                                    </dl>

                                    {if s.not_received_count > 0 {
                                        view! {
                                            <div class="alert">
                                                <strong>
                                                    "Не отримано / Not received: "
                                                    {s.not_received_count}
                                                </strong>
                                            </div>
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

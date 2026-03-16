use leptos::prelude::*;

// ── Types ─────────────────────────────────────────────────────────────────────

/// All possible states a participant's home page can show.
///
/// One enum, one match. No scattered conditionals.
/// Derives from (season phase + participant enrollment + confirmation + assignment state).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HomeState {
    /// No active launched season.
    NoSeason,

    /// Enrollment phase, participant is NOT enrolled.
    EnrollmentOpen {
        deadline: String,
        theme: Option<String>,
    },

    /// Enrollment phase, participant IS enrolled.
    Enrolled { confirm_deadline: String },

    /// Preparation phase, participant is enrolled but NOT confirmed.
    Preparing { confirm_deadline: String },

    /// Preparation phase, participant has confirmed.
    Confirmed,

    /// Assignment phase — organizer is preparing assignments.
    Assigning,

    /// Delivery phase, assignment exists, receipt not yet confirmed.
    Assigned {
        recipient_name: String,
        recipient_phone: String,
        recipient_city: String,
        recipient_branch_number: i32,
    },

    /// Delivery phase, participant has confirmed receipt (received or not received).
    ReceiptConfirmed,

    /// Season is complete.
    Complete,
}

// ── SSR-only row types ─────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
struct SeasonInfoRow {
    id: uuid::Uuid,
    phase: crate::types::Phase,
    signup_deadline: time::OffsetDateTime,
    confirm_deadline: time::OffsetDateTime,
    theme: Option<String>,
}

#[cfg(feature = "ssr")]
struct EnrollSeasonRow {
    id: uuid::Uuid,
    signup_deadline: time::OffsetDateTime,
}

#[cfg(feature = "ssr")]
struct ConfirmSeasonRow {
    id: uuid::Uuid,
    confirm_deadline: time::OffsetDateTime,
}

#[cfg(feature = "ssr")]
struct AssignmentRow {
    recipient_name: String,
    recipient_phone: String,
    nova_poshta_city: String,
    nova_poshta_number: i32,
    receipt_status: crate::types::ReceiptStatus,
}

// ── SSR helpers ────────────────────────────────────────────────────────────────

/// Resolve home state for the Enrollment phase.
#[cfg(feature = "ssr")]
async fn resolve_enrollment_state(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    season: &SeasonInfoRow,
    signup_str: String,
    confirm_str: String,
) -> Result<HomeState, ServerFnError> {
    let enrolled = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM enrollments
            WHERE user_id = $1 AND season_id = $2
        ) AS "exists!"
        "#,
        user_id,
        season.id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if enrolled {
        Ok(HomeState::Enrolled {
            confirm_deadline: confirm_str,
        })
    } else {
        Ok(HomeState::EnrollmentOpen {
            deadline: signup_str,
            theme: season.theme.clone(),
        })
    }
}

/// Resolve home state for the Preparation phase.
#[cfg(feature = "ssr")]
async fn resolve_preparation_state(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    season: &SeasonInfoRow,
    confirm_str: String,
) -> Result<HomeState, ServerFnError> {
    let confirmed = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM enrollments
            WHERE user_id = $1 AND season_id = $2
              AND confirmed_ready_at IS NOT NULL
        ) AS "exists!"
        "#,
        user_id,
        season.id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if confirmed {
        Ok(HomeState::Confirmed)
    } else {
        Ok(HomeState::Preparing {
            confirm_deadline: confirm_str,
        })
    }
}

/// Resolve home state for the Delivery phase.
#[cfg(feature = "ssr")]
async fn resolve_delivery_state(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    season_id: uuid::Uuid,
) -> Result<HomeState, ServerFnError> {
    use crate::types::ReceiptStatus;

    let assignment = sqlx::query_as!(
        AssignmentRow,
        r#"
        SELECT
            u.name AS recipient_name,
            u.phone AS recipient_phone,
            da.nova_poshta_city,
            da.nova_poshta_number,
            a.receipt_status AS "receipt_status: ReceiptStatus"
        FROM assignments a
        JOIN users u ON u.id = a.recipient_id
        JOIN delivery_addresses da ON da.user_id = a.recipient_id
        WHERE a.sender_id = $1 AND a.season_id = $2
        "#,
        user_id,
        season_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    let Some(a) = assignment else {
        return Ok(HomeState::Assigning);
    };

    match a.receipt_status {
        ReceiptStatus::NoResponse => Ok(HomeState::Assigned {
            recipient_name: a.recipient_name,
            recipient_phone: a.recipient_phone,
            recipient_city: a.nova_poshta_city,
            recipient_branch_number: a.nova_poshta_number,
        }),
        ReceiptStatus::Received | ReceiptStatus::NotReceived => Ok(HomeState::ReceiptConfirmed),
    }
}

// ── Server functions ───────────────────────────────────────────────────────────

/// Compute the home page state for the authenticated participant.
///
/// # Errors
///
/// Returns `Err` if session is invalid or DB fails.
#[server]
pub async fn get_home_state() -> Result<HomeState, ServerFnError> {
    use crate::{auth, error::AppError, types::Phase};
    use http::request::Parts;
    use time::format_description::well_known::Rfc3339;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    let season = sqlx::query_as!(
        SeasonInfoRow,
        r#"
        SELECT id, phase AS "phase: Phase", signup_deadline, confirm_deadline, theme
        FROM seasons
        WHERE phase NOT IN ('complete', 'cancelled')
          AND launched_at IS NOT NULL
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    let Some(season) = season else {
        let has_complete = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM seasons WHERE phase = 'complete') AS "exists!""#,
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

        return if has_complete {
            Ok(HomeState::Complete)
        } else {
            Ok(HomeState::NoSeason)
        };
    };

    let signup_str = season.signup_deadline.format(&Rfc3339).unwrap_or_default();
    let confirm_str = season.confirm_deadline.format(&Rfc3339).unwrap_or_default();

    match season.phase {
        Phase::Enrollment => {
            resolve_enrollment_state(&pool, user.id, &season, signup_str, confirm_str).await
        }
        Phase::Preparation => resolve_preparation_state(&pool, user.id, &season, confirm_str).await,
        Phase::Assignment => Ok(HomeState::Assigning),
        Phase::Delivery => resolve_delivery_state(&pool, user.id, season.id).await,
        Phase::Complete | Phase::Cancelled => Ok(HomeState::Complete),
    }
}

/// Enroll the current participant in the active season.
///
/// Also UPSERTs the Nova Poshta delivery address if provided.
/// Enrollment requires a delivery address to exist (set during onboarding or here).
///
/// # Errors
///
/// Returns `Err` if not logged in, season not in Enrollment phase, deadline passed,
/// no delivery address, or already enrolled.
#[server]
pub async fn enroll_in_season(branch: String) -> Result<(), ServerFnError> {
    use crate::{auth, error::AppError};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    let season = sqlx::query_as!(
        EnrollSeasonRow,
        r#"
        SELECT id, signup_deadline
        FROM seasons
        WHERE phase = 'enrollment'
          AND launched_at IS NOT NULL
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("enrollment is not open"))?;

    // Deadline check — bypassed in test mode
    let test_mode = std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true");
    if !test_mode && season.signup_deadline < time::OffsetDateTime::now_utc() {
        return Err(ServerFnError::new("enrollment deadline has passed"));
    }

    // UPSERT delivery address from the branch field
    let trimmed = branch.trim().to_owned();
    if !trimmed.is_empty() {
        let number: i32 = trimmed
            .chars()
            .skip_while(|c: &char| !c.is_ascii_digit())
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .unwrap_or(1);

        sqlx::query!(
            r#"
            INSERT INTO delivery_addresses (user_id, nova_poshta_city, nova_poshta_number)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) DO UPDATE
                SET nova_poshta_city = EXCLUDED.nova_poshta_city,
                    nova_poshta_number = EXCLUDED.nova_poshta_number,
                    updated_at = now()
            "#,
            user.id,
            trimmed,
            number,
        )
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;
    }

    // Verify user has a delivery address
    let has_address = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM delivery_addresses WHERE user_id = $1
        ) AS "exists!"
        "#,
        user.id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    if !has_address {
        return Err(ServerFnError::new(
            "complete onboarding first to set your delivery address",
        ));
    }

    sqlx::query!(
        r#"
        INSERT INTO enrollments (user_id, season_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, season_id) DO NOTHING
        "#,
        user.id,
        season.id,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(())
}

/// Confirm the participant is ready for assignment.
///
/// Sets `confirmed_ready_at = now()` on their enrollment row.
/// One-way latch: once confirmed, cannot be un-confirmed.
///
/// # Errors
///
/// Returns `Err` if not in Preparation phase, deadline passed, or not enrolled.
#[server]
pub async fn confirm_ready() -> Result<(), ServerFnError> {
    use crate::{auth, error::AppError};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(AppError::into_server_fn_error)?;

    let season = sqlx::query_as!(
        ConfirmSeasonRow,
        r#"
        SELECT id, confirm_deadline
        FROM seasons
        WHERE phase = 'preparation'
          AND launched_at IS NOT NULL
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("confirmation is not open"))?;

    // Deadline check — bypassed in test mode
    let test_mode = std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true");
    if !test_mode && season.confirm_deadline < time::OffsetDateTime::now_utc() {
        return Err(ServerFnError::new("confirmation deadline has passed"));
    }

    // Idempotent — if already confirmed, rows_affected == 0, that's fine
    sqlx::query!(
        r#"
        UPDATE enrollments
        SET confirmed_ready_at = now()
        WHERE user_id = $1 AND season_id = $2
          AND confirmed_ready_at IS NULL
        "#,
        user.id,
        season.id,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(())
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Home page for authenticated, onboarded participants.
///
/// Single component, single match on `HomeState`. No scattered conditionals.
#[component]
pub fn HomePage() -> impl IntoView {
    let enroll_action = ServerAction::<EnrollInSeason>::new();
    let confirm_action = ServerAction::<ConfirmReady>::new();

    // Refetch home state after any action
    let home_state = Resource::new(
        move || {
            (
                enroll_action.version().get(),
                confirm_action.version().get(),
            )
        },
        |_| get_home_state(),
    );

    // Hydration gate
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

    view! {
        <div class="home-page">
            // Action error display
            {move || {
                let err = enroll_action.value().get().and_then(Result::err)
                    .or_else(|| confirm_action.value().get().and_then(Result::err));
                err.map(|e| view! {
                    <p class="error">{e.to_string()}</p>
                })
            }}

            <Suspense fallback=|| view! { <p>"Завантаження..."</p> }>
                {move || home_state.get().map(|result| match result {
                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                    Ok(state) => render_home_state(state, enroll_action, confirm_action, hydrated),
                })}
            </Suspense>
        </div>
    }
}

// Rendering dispatch — each arm is a distinct UI state with its own markup.
// Splitting further would add indirection without reducing complexity.
#[allow(clippy::too_many_lines)]
fn render_home_state(
    state: HomeState,
    enroll_action: ServerAction<EnrollInSeason>,
    confirm_action: ServerAction<ConfirmReady>,
    hydrated: ReadSignal<bool>,
) -> AnyView {
    match state {
        HomeState::NoSeason => view! {
            <p>
                "No season is currently active. / Немає активного сезону. "
                "You'll receive an SMS when a new season opens."
            </p>
        }
        .into_any(),

        HomeState::EnrollmentOpen { deadline, theme } => view! {
            <h2>"Відкрита реєстрація / Enrollment Open"</h2>

            {theme.as_ref().map(|t| view! {
                <p class="theme">"Тема / Theme: " {t.clone()}</p>
            })}

            <p class="deadline">"Реєстрація до / Deadline: " {deadline}</p>

            <p class="guideline">
                "Self-expression / Самовираження: "
                "Надішліть щось, що відображає вас."
            </p>

            <leptos::form::ActionForm action=enroll_action>
                <div>
                    <label for="branch-enroll">
                        "Nova Poshta відділення (branch) — оновити при потребі"
                    </label>
                    <input
                        id="branch-enroll"
                        type="text"
                        name="branch"
                        placeholder="Відділення №1, Київ"
                    />
                </div>
                <button
                    type="submit"
                    data-testid="enroll-button"
                    disabled=move || !hydrated.get()
                >
                    "Зареєструватись / Enroll"
                </button>
            </leptos::form::ActionForm>
        }
        .into_any(),

        HomeState::Enrolled { confirm_deadline } => view! {
            <h2>"Ви зареєстровані / You are enrolled"</h2>
            <p>
                "Enrollment confirmed. Create your mail. / "
                "Реєстрація підтверджена. Створіть свій лист."
            </p>
            <p class="deadline">"Дедлайн підтвердження / Confirm deadline: "
                {confirm_deadline}
            </p>
        }
        .into_any(),

        HomeState::Preparing { confirm_deadline } => view! {
            <h2>"Підготовка / Preparation"</h2>
            <p>
                "Create your mail / Створіть свій лист. "
                "Confirm ready before the deadline / підтвердьте готовність до дедлайну."
            </p>
            <p class="deadline">"Дедлайн / Deadline: " {confirm_deadline}</p>

            <leptos::form::ActionForm action=confirm_action>
                <button
                    type="submit"
                    data-testid="confirm-ready-button"
                    disabled=move || !hydrated.get()
                >
                    "Підтвердити готовність / Confirm Ready"
                </button>
            </leptos::form::ActionForm>
        }
        .into_any(),

        HomeState::Confirmed => view! {
            <h2>"Готовність підтверджена / Ready Confirmed"</h2>
            <p>
                "Your mail is confirmed. / Ваш лист підтверджено. "
                "Очікуйте на розподіл."
            </p>
        }
        .into_any(),

        HomeState::Assigning => view! {
            <h2>"Розподіл / Assignment"</h2>
            <p>
                "The organizer / організатор is preparing assignments. "
                "Зачекайте — скоро отримаєте повідомлення."
            </p>
        }
        .into_any(),

        HomeState::Assigned {
            recipient_name,
            recipient_phone,
            recipient_city,
            recipient_branch_number,
        } => view! {
            <h2>"Ваш отримувач / Your recipient"</h2>
            <p>"Arriving / Отримання: your parcel is on its way. Confirm receipt when it arrives."</p>

            <dl>
                <dt>"Ім'я / Name"</dt>
                <dd data-testid="recipient-name">{recipient_name}</dd>

                <dt>"Телефон / Phone"</dt>
                <dd data-testid="recipient-phone">{recipient_phone}</dd>

                <dt>"Nova Poshta"</dt>
                <dd data-testid="recipient-branch">
                    {format!("Відділення №{recipient_branch_number}, {recipient_city}")}
                </dd>
            </dl>
        }
        .into_any(),

        HomeState::ReceiptConfirmed => view! {
            <h2>"Дякуємо! / Thanks!"</h2>
            <p>"Receipt confirmed / Отримання підтверджено."</p>
        }
        .into_any(),

        HomeState::Complete => view! {
            <h2>"Сезон завершено / Season Complete"</h2>
            <p>
                "This season is complete. / Цей сезон завершено. "
                "Дякуємо за участь!"
            </p>
        }
        .into_any(),
    }
}

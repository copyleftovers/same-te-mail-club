use crate::components::toast::use_toast;
use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

#[cfg(not(feature = "ssr"))]
use leptos::web_sys;

#[cfg(feature = "ssr")]
use crate::error::db_err;

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
    .map_err(db_err)?;

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
    .map_err(db_err)?;

    if confirmed {
        Ok(HomeState::Confirmed)
    } else {
        Ok(HomeState::Preparing {
            confirm_deadline: confirm_str,
        })
    }
}

/// Resolve home state for the Delivery phase.
///
/// Two independent queries:
/// 1. Outgoing assignment (`sender_id` = user) — provides recipient details to display.
/// 2. Incoming assignment (`recipient_id` = user) — tracks whether the user has confirmed
///    their own receipt. These are different rows and must not be conflated.
#[cfg(feature = "ssr")]
async fn resolve_delivery_state(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    season_id: uuid::Uuid,
) -> Result<HomeState, ServerFnError> {
    use crate::types::ReceiptStatus;

    // Outgoing assignment: who does this user need to send to?
    let outgoing = sqlx::query_as!(
        AssignmentRow,
        r#"
        SELECT
            u.name AS recipient_name,
            u.phone AS recipient_phone,
            da.nova_poshta_city,
            da.nova_poshta_number
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
    .map_err(db_err)?;

    // No outgoing assignment yet — organizer is still in the assignment phase.
    let Some(a) = outgoing else {
        return Ok(HomeState::Assigning);
    };

    // Incoming assignment: has this user confirmed receiving their own mail?
    let incoming_status = sqlx::query_scalar!(
        r#"
        SELECT receipt_status AS "receipt_status: ReceiptStatus"
        FROM assignments
        WHERE recipient_id = $1 AND season_id = $2
        "#,
        user_id,
        season_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(db_err)?;

    match incoming_status {
        Some(ReceiptStatus::Received | ReceiptStatus::NotReceived) => {
            Ok(HomeState::ReceiptConfirmed)
        }
        _ => Ok(HomeState::Assigned {
            recipient_name: a.recipient_name,
            recipient_phone: a.recipient_phone,
            recipient_city: a.nova_poshta_city,
            recipient_branch_number: a.nova_poshta_number,
        }),
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
    use crate::{auth, date_format::format_date_uk, types::Phase};

    let (pool, user) = auth::require_auth().await?;

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
    .map_err(db_err)?;

    let Some(season) = season else {
        // Show Complete only when the most recently created season is Complete.
        // A Cancelled season means the organizer abandoned it — the participant
        // should see "no season / SMS pending", not the completion message.
        let most_recent_phase = sqlx::query_scalar!(
            r#"
            SELECT phase AS "phase: Phase"
            FROM seasons
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&pool)
        .await
        .map_err(db_err)?;

        return match most_recent_phase {
            Some(Phase::Complete) => Ok(HomeState::Complete),
            _ => Ok(HomeState::NoSeason),
        };
    };

    let signup_str = format_date_uk(season.signup_deadline);
    let confirm_str = format_date_uk(season.confirm_deadline);

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
pub async fn enroll_in_season(city: String, np_number: String) -> Result<(), ServerFnError> {
    use crate::auth;

    let (pool, user) = auth::require_auth().await?;

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
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("enrollment is not open"))?;

    // Deadline check — bypassed in test mode
    let test_mode = std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true");
    if !test_mode && season.signup_deadline < time::OffsetDateTime::now_utc() {
        return Err(ServerFnError::new("enrollment deadline has passed"));
    }

    // UPSERT delivery address from the split fields
    let city = city.trim().to_owned();
    let number: i32 = if np_number.trim().is_empty() {
        // If no number provided but city is, skip UPSERT (use existing address)
        0
    } else {
        np_number
            .trim()
            .parse()
            .map_err(|_| ServerFnError::new("invalid branch number"))?
    };

    if !city.is_empty() && number > 0 {
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
            city,
            number,
        )
        .execute(&pool)
        .await
        .map_err(db_err)?;
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
    .map_err(db_err)?;

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
    .map_err(db_err)?;

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
    use crate::auth;

    let (pool, user) = auth::require_auth().await?;

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
    .map_err(db_err)?
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
    .map_err(db_err)?;

    Ok(())
}

/// Confirm the participant has received (or not received) their mail.
///
/// `received` is passed as a string "true"/"false" from the HTML form (submit button value).
/// One-way latch: once confirmed, cannot be changed.
/// The participant is the RECIPIENT — this query uses `recipient_id`.
///
/// # Errors
///
/// Returns `Err` if not logged in, season not in Delivery phase, or already confirmed.
#[server]
pub async fn confirm_receipt(received: String, note: Option<String>) -> Result<(), ServerFnError> {
    use crate::{auth, types::ReceiptStatus};

    let (pool, user) = auth::require_auth().await?;

    // Parse received from "true"/"false" string (HTML form submit button value)
    let received_bool = match received.as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(ServerFnError::new(format!(
                "invalid received value: {other}"
            )));
        }
    };

    // Get active season in Delivery phase
    let season_id = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM seasons
        WHERE phase = 'delivery'
          AND launched_at IS NOT NULL
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("no active delivery season"))?;

    // Find assignment where this user is the RECIPIENT
    let assignment_id = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM assignments
        WHERE recipient_id = $1 AND season_id = $2
          AND receipt_status = 'no_response'
        "#,
        user.id,
        season_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("already confirmed or no assignment found"))?;

    let new_status = if received_bool {
        ReceiptStatus::Received
    } else {
        ReceiptStatus::NotReceived
    };

    // Normalize empty string to None
    let note = note.and_then(|n| {
        let t = n.trim().to_owned();
        if t.is_empty() { None } else { Some(t) }
    });

    sqlx::query!(
        r#"
        UPDATE assignments
        SET receipt_status = $1, receipt_note = $2
        WHERE id = $3
        "#,
        new_status as ReceiptStatus,
        note,
        assignment_id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?;

    Ok(())
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Home page for authenticated, onboarded participants.
///
/// Single component, single match on `HomeState`. No scattered conditionals.
#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();
    let enroll_action = ServerAction::<EnrollInSeason>::new();
    let confirm_action = ServerAction::<ConfirmReady>::new();
    let receipt_action = ServerAction::<ConfirmReceipt>::new();

    // Refetch home state after any action
    let home_state = Resource::new(
        move || {
            (
                enroll_action.version().get(),
                confirm_action.version().get(),
                receipt_action.version().get(),
            )
        },
        |_| get_home_state(),
    );

    let hydrated = use_hydrated();
    let set_toast = use_toast();

    // Toast feedback for successful actions
    Effect::new(move |_| {
        if let Some(Ok(())) = enroll_action.value().get() {
            set_toast.set(Some("Записано на сезон!".into()));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = confirm_action.value().get() {
            set_toast.set(Some("Готовність підтверджено!".into()));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = receipt_action.value().get() {
            set_toast.set(Some("Дякуємо за підтвердження!".into()));
        }
    });

    view! {
        <div class="prose-page">
            // Action error display
            <div role="alert" aria-live="assertive" data-testid="action-error">
                {move || {
                    let err = enroll_action
                        .value()
                        .get()
                        .and_then(Result::err)
                        .or_else(|| confirm_action.value().get().and_then(Result::err))
                        .or_else(|| receipt_action.value().get().and_then(Result::err));
                    err.map(|e| view! { <p class="alert">{e.to_string()}</p> })
                }}
            </div>

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
                    home_state
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="alert">{e.to_string()}</p> }.into_any(),
                            Ok(state) => {
                                render_home_state(
                                    state,
                                    enroll_action,
                                    confirm_action,
                                    receipt_action,
                                    hydrated,
                                    i18n,
                                )
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

fn render_enrollment_open(
    deadline: String,
    theme: Option<&String>,
    enroll_action: ServerAction<EnrollInSeason>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    let pending = enroll_action.pending();
    view! {
        <h2>{t!(i18n, home_enroll_open_heading)}</h2>

        {theme
            .map(|theme_val| {
                view! {
                    <p class="theme" data-testid="season-theme">
                        {t!(i18n, home_theme_label)}
                        {theme_val.clone()}
                    </p>
                }
            })}

        <p class="deadline">{t!(i18n, home_signup_deadline_label)} {deadline}</p>

        <p class="guideline">{t!(i18n, home_guideline)}</p>

        <leptos::form::ActionForm action=enroll_action>
            <div class="flex flex-col gap-(--density-space-md) sm:flex-row sm:gap-(--density-space-sm)">
                <div class="field sm:w-1/2">
                    <label class="field-label" for="enroll-city">
                        {t!(i18n, home_city_label)}
                    </label>
                    <input
                        class="field-input"
                        type="text"
                        id="enroll-city"
                        name="city"
                        value="Київ"
                        data-testid="np-city-input"
                    />
                </div>
                <div class="field sm:w-1/2">
                    <label class="field-label" for="enroll-number">
                        {t!(i18n, home_np_number_label)}
                    </label>
                    <input
                        class="field-input"
                        id="enroll-number"
                        type="text"
                        inputmode="numeric"
                        name="np_number"
                        placeholder="123"
                        data-testid="np-number-input"
                    />
                </div>
            </div>
            <button
                class="btn"
                type="submit"
                data-testid="enroll-button"
                disabled=move || pending.get() || !hydrated.get()
            >
                {move || if pending.get() {
                    "Записуюсь...".into_any()
                } else {
                    t!(i18n, home_enroll_button).into_any()
                }}
            </button>
        </leptos::form::ActionForm>
    }
    .into_any()
}

fn render_assigned_view(
    recipient_name: &str,
    recipient_phone: &str,
    recipient_city: &str,
    recipient_branch_number: i32,
    receipt_action: ServerAction<ConfirmReceipt>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    render_envelope_reveal(
        recipient_name,
        recipient_phone,
        recipient_city,
        recipient_branch_number,
        receipt_action,
        hydrated,
        i18n,
    )
}

fn render_receipt_form(
    receipt_action: ServerAction<ConfirmReceipt>,
    receipt_pending: Memo<bool>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> impl IntoView {
    view! {
        <section class="receipt-section">
            <h3>{t!(i18n, home_confirm_receipt_heading)}</h3>

            <leptos::form::ActionForm action=receipt_action>
                <div class="field">
                    <label class="field-label" for="receipt-note">
                        {t!(i18n, home_receipt_note_label)}
                    </label>
                    <textarea
                        class="field-input"
                        id="receipt-note"
                        name="note"
                        placeholder=move || t_string!(i18n, home_receipt_note_placeholder)
                        data-testid="receipt-note-input"
                        aria-invalid=move || {
                            receipt_action.value().get().and_then(Result::err).is_some()
                        }
                        aria-describedby="action-error"
                    ></textarea>
                </div>
                <button
                    class="btn"
                    type="submit"
                    name="received"
                    value="true"
                    data-testid="received-button"
                    disabled=move || receipt_pending.get() || !hydrated.get()
                >
                    {move || if receipt_pending.get() {
                        "Надсилаю...".into_any()
                    } else {
                        t!(i18n, home_received_button).into_any()
                    }}
                </button>
                <button
                    class="btn"
                    data-variant="secondary"
                    type="submit"
                    name="received"
                    value="false"
                    data-testid="not-received-button"
                    disabled=move || receipt_pending.get() || !hydrated.get()
                >
                    {move || if receipt_pending.get() {
                        "Надсилаю...".into_any()
                    } else {
                        t!(i18n, home_not_received_button).into_any()
                    }}
                </button>
            </leptos::form::ActionForm>
        </section>
    }
}

fn setup_envelope_reveal() -> (
    ReadSignal<bool>,
    WriteSignal<bool>,
    ReadSignal<bool>,
    WriteSignal<bool>,
) {
    let (revealed, set_revealed) = signal(false);
    let (confetti_active, set_confetti_active) = signal(false);

    // Check localStorage for already_seen state on client-side only
    Effect::new(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            if let Some(window) = web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
            {
                let key = "assignment_revealed";
                if let Ok(Some(_)) = storage.get_item(key) {
                    set_revealed.set(true);
                }
            }
        }
    });

    (revealed, set_revealed, confetti_active, set_confetti_active)
}

fn render_envelope_reveal(
    recipient_name: &str,
    recipient_phone: &str,
    recipient_city: &str,
    recipient_branch_number: i32,
    receipt_action: ServerAction<ConfirmReceipt>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    let receipt_pending = receipt_action.pending();
    let (revealed, set_revealed, confetti_active, set_confetti_active) = setup_envelope_reveal();

    let on_reveal = move |()| {
        if !revealed.get_untracked() {
            set_revealed.set(true);
            set_confetti_active.set(true);

            // Save to localStorage
            #[cfg(not(feature = "ssr"))]
            {
                if let Some(window) = web_sys::window()
                    && let Ok(Some(storage)) = window.local_storage()
                {
                    let key = "assignment_revealed";
                    let _ = storage.set_item(key, "true");
                }
            }

            // Confetti auto-dismiss after 1.5s
            leptos::prelude::set_timeout(
                move || set_confetti_active.set(false),
                std::time::Duration::from_millis(1500),
            );
        }
    };

    let on_keydown = {
        #[cfg(not(feature = "ssr"))]
        {
            move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Enter" || ev.key() == " " {
                    ev.prevent_default();
                    on_reveal(());
                }
            }
        }
        #[cfg(feature = "ssr")]
        {
            move |_| {}
        }
    };

    let recipient_branch_text = t!(
        i18n,
        home_recipient_branch,
        branch_number = recipient_branch_number,
        city = recipient_city.to_string()
    );

    view! {
        <h2>{t!(i18n, home_assigned_heading)}</h2>
        <p>{t!(i18n, home_send_instructions)}</p>

        <article
            class="reveal-envelope"
            data-testid="reveal-envelope"
            role="button"
            tabindex="0"
            attr:aria-expanded=move || if revealed.get() { "true" } else { "false" }
            attr:aria-label="Tap to reveal your recipient"
            on:click=move |_| on_reveal(())
            on:keydown=on_keydown
        >
            <div class="envelope-body" aria-hidden="true">
                <div class="envelope-flap"></div>
            </div>
            <div class="envelope-card">
                <h3 class="envelope-recipient" data-testid="recipient-name">
                    {recipient_name.to_string()}
                </h3>
                <dl class="info-list">
                    <div class="info-item">
                        <dt class="info-label">{t!(i18n, home_branch_label)}</dt>
                        <dd class="info-value" data-testid="recipient-branch">
                            {recipient_branch_text}
                        </dd>
                    </div>
                    <div class="info-item">
                        <dt class="info-label">{t!(i18n, home_phone_label)}</dt>
                        <dd class="info-value">
                            <a
                                href=format!("tel:{}", recipient_phone.to_string())
                                class="info-link"
                                data-testid="recipient-phone"
                            >
                                {recipient_phone.to_string()}
                            </a>
                        </dd>
                    </div>
                </dl>
            </div>
        </article>

        <div class="confetti" attr:data-active=move || if confetti_active.get() { "true" } else { "false" } aria-hidden="true"></div>

        {render_receipt_form(receipt_action, receipt_pending, hydrated, i18n)}
    }
    .into_any()
}

fn render_home_state(
    state: HomeState,
    enroll_action: ServerAction<EnrollInSeason>,
    confirm_action: ServerAction<ConfirmReady>,
    receipt_action: ServerAction<ConfirmReceipt>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    match state {
        HomeState::NoSeason => view! {
            <div class="empty-state">
                <p class="empty-state-headline">{t!(i18n, home_no_season)}</p>
            </div>
        }
        .into_any(),

        HomeState::EnrollmentOpen { deadline, theme } => {
            render_enrollment_open(deadline, theme.as_ref(), enroll_action, hydrated, i18n)
        }

        HomeState::Enrolled { confirm_deadline } => view! {
            <h2>{t!(i18n, home_enrolled_heading)}</h2>
            <p>{t!(i18n, home_enrolled_desc)}</p>
            <p class="deadline">{t!(i18n, home_confirm_deadline_label)} {confirm_deadline}</p>
        }
        .into_any(),

        HomeState::Preparing { confirm_deadline } => {
            let confirm_pending = confirm_action.pending();
            view! {
                <h2>{t!(i18n, home_preparing_heading)}</h2>
                <p>{t!(i18n, home_preparing_body)}</p>
                <p class="deadline" data-testid="season-deadline">
                    {t!(i18n, home_deadline_label)}
                    {confirm_deadline}
                </p>

                <leptos::form::ActionForm action=confirm_action>
                    <button
                        class="btn"
                        type="submit"
                        data-testid="confirm-ready-button"
                        disabled=move || confirm_pending.get() || !hydrated.get()
                    >
                        {move || if confirm_pending.get() {
                            "Підтверджую...".into_any()
                        } else {
                            t!(i18n, home_confirm_ready_button).into_any()
                        }}
                    </button>
                </leptos::form::ActionForm>
            }
            .into_any()
        }

        HomeState::Confirmed => view! {
            <h2>{t!(i18n, home_ready_confirmed_heading)}</h2>
            <p>{t!(i18n, home_waiting_assignment)}</p>
        }
        .into_any(),

        HomeState::Assigning => view! {
            <h2>{t!(i18n, home_assigning_heading)}</h2>
            <p>{t!(i18n, home_assigning_desc)}</p>
        }
        .into_any(),

        HomeState::Assigned {
            recipient_name,
            recipient_phone,
            recipient_city,
            recipient_branch_number,
        } => render_assigned_view(
            &recipient_name,
            &recipient_phone,
            &recipient_city,
            recipient_branch_number,
            receipt_action,
            hydrated,
            i18n,
        ),

        HomeState::ReceiptConfirmed => view! {
            <h2>{t!(i18n, home_thanks_heading)}</h2>
            <p data-testid="receipt-thanks">{t!(i18n, home_reported_label)}</p>
        }
        .into_any(),

        HomeState::Complete => view! {
            <h2>{t!(i18n, home_complete_heading)}</h2>
            <p>{t!(i18n, home_thanks_participation)}</p>
        }
        .into_any(),
    }
}

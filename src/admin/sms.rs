use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, use_i18n};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Summary of an SMS batch send operation.
///
/// Returned to the admin after triggering any SMS batch.
/// `sent` + `failed` = total recipients targeted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsReport {
    pub sent: u32,
    pub failed: u32,
    pub failed_phones: Vec<String>,
}

/// SSR-only row type for assignment notification targets.
#[cfg(feature = "ssr")]
struct AssignmentTarget {
    id: uuid::Uuid,
    phone: String,
}

// ── Server functions ───────────────────────────────────────────────────────────

/// Story 5.3: Season-open notification to ALL active users.
///
/// Sent when the organizer launches a season. Targets every active account,
/// not just enrolled participants.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_season_open_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;

    let phones = sqlx::query_scalar!(r#"SELECT phone FROM users WHERE status = 'active'"#,)
        .fetch_all(&pool)
        .await
        .map_err(db_err)?;

    let message = td_string!(Locale::uk, sms_season_open_body);

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for phone in phones {
        match sms::send_sms(&config, &phone, message).await {
            Ok(()) => sent += 1,
            Err(e) => {
                tracing::warn!(phone = %phone, error = %e, "SMS send failed");
                failed_phones.push(phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

/// Story 5.1: Assignment notification — sets `notified_at` per-assignment on success.
///
/// Sent to senders whose `notified_at IS NULL` (haven't been notified yet).
/// On failure, leaves `notified_at` NULL so the organizer can see who wasn't reached.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_assignment_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;

    // Assignment season required (assignment or delivery phase — both have assignments)
    let season_id = sqlx::query_scalar!(
        r#"SELECT id FROM seasons WHERE phase IN ('assignment', 'delivery') AND launched_at IS NOT NULL"#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("no active assignment/delivery season"))?;

    // Senders who haven't been notified yet
    let targets = sqlx::query_as!(
        AssignmentTarget,
        r#"
        SELECT a.id, u.phone
        FROM assignments a
        JOIN users u ON u.id = a.sender_id
        WHERE a.season_id = $1 AND a.notified_at IS NULL
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let message = td_string!(Locale::uk, sms_assignment_body);

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for target in targets {
        match sms::send_sms(&config, &target.phone, message).await {
            Ok(()) => {
                // Mark notified_at on success; leave NULL on failure
                let _ = sqlx::query!(
                    r#"UPDATE assignments SET notified_at = now() WHERE id = $1"#,
                    target.id,
                )
                .execute(&pool)
                .await;
                sent += 1;
            }
            Err(e) => {
                tracing::warn!(phone = %target.phone, error = %e, "SMS send failed");
                failed_phones.push(target.phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

/// Story 5.4: Pre-deadline nudge to enrolled users who haven't confirmed ready.
///
/// Targets enrolled users with `confirmed_ready_at IS NULL`.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_confirm_nudge_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;

    // Active preparation season required
    let season_id = sqlx::query_scalar!(
        r#"SELECT id FROM seasons WHERE phase = 'preparation' AND launched_at IS NOT NULL"#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("no active preparation season"))?;

    let confirm_deadline = sqlx::query_scalar!(
        r#"SELECT confirm_deadline FROM seasons WHERE id = $1"#,
        season_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    let deadline_str = crate::date_format::format_date_uk(confirm_deadline);

    let phones = sqlx::query_scalar!(
        r#"
        SELECT u.phone
        FROM enrollments e
        JOIN users u ON u.id = e.user_id
        WHERE e.season_id = $1 AND e.confirmed_ready_at IS NULL
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let prefix = td_string!(Locale::uk, sms_confirm_nudge_body_prefix);
    let message = format!("{prefix}{deadline_str}.");

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for phone in phones {
        match sms::send_sms(&config, &phone, &message).await {
            Ok(()) => sent += 1,
            Err(e) => {
                tracing::warn!(phone = %phone, error = %e, "SMS send failed");
                failed_phones.push(phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

/// Story 5.2: Receipt nudge to recipients with `receipt_status = 'no_response'`.
///
/// Sent after assignments have been delivered to prompt confirmation.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn send_receipt_nudge_sms() -> Result<SmsReport, ServerFnError> {
    use crate::{
        auth,
        config::Config,
        i18n::i18n::{Locale, td_string},
        sms,
    };

    let (pool, _user) = auth::require_admin().await?;
    let config = leptos::context::use_context::<Config>()
        .ok_or_else(|| ServerFnError::new("no config in context"))?;

    // Active delivery season required
    let season_id = sqlx::query_scalar!(
        r#"SELECT id FROM seasons WHERE phase = 'delivery' AND launched_at IS NOT NULL"#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(|| ServerFnError::new("no active delivery season"))?;

    let phones = sqlx::query_scalar!(
        r#"
        SELECT u.phone
        FROM assignments a
        JOIN users u ON u.id = a.recipient_id
        WHERE a.season_id = $1 AND a.receipt_status = 'no_response'
        "#,
        season_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;

    let message = td_string!(Locale::uk, sms_receipt_nudge_body);

    let mut sent: u32 = 0;
    let mut failed_phones: Vec<String> = Vec::new();

    for phone in phones {
        match sms::send_sms(&config, &phone, message).await {
            Ok(()) => sent += 1,
            Err(e) => {
                tracing::warn!(phone = %phone, error = %e, "SMS send failed");
                failed_phones.push(phone);
            }
        }
    }

    let failed = u32::try_from(failed_phones.len()).unwrap_or(u32::MAX);
    Ok(SmsReport {
        sent,
        failed,
        failed_phones,
    })
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Admin SMS batch trigger page.
///
/// Each button fires one batch SMS job; result shown in `sms-report`.
/// Uses four separate actions so the result of each is tracked independently.
// Four independent ActionForms each with their own trigger section.
// Splitting into sub-components adds call-site indirection without reducing entanglement —
// the actions, hydration gate, and report display are all coupled to each other.
#[allow(clippy::too_many_lines)]
#[component]
pub fn SmsPage() -> impl IntoView {
    let i18n = use_i18n();
    let season_open_action = ServerAction::<SendSeasonOpenSms>::new();
    let assignment_action = ServerAction::<SendAssignmentSms>::new();
    let confirm_nudge_action = ServerAction::<SendConfirmNudgeSms>::new();
    let receipt_nudge_action = ServerAction::<SendReceiptNudgeSms>::new();

    // Show the most recent SMS report (whichever action completed last)
    let latest_report = move || {
        season_open_action
            .value()
            .get()
            .and_then(Result::ok)
            .or_else(|| assignment_action.value().get().and_then(Result::ok))
            .or_else(|| confirm_nudge_action.value().get().and_then(Result::ok))
            .or_else(|| receipt_nudge_action.value().get().and_then(Result::ok))
    };

    let latest_error = move || {
        season_open_action
            .value()
            .get()
            .and_then(Result::err)
            .or_else(|| assignment_action.value().get().and_then(Result::err))
            .or_else(|| confirm_nudge_action.value().get().and_then(Result::err))
            .or_else(|| receipt_nudge_action.value().get().and_then(Result::err))
    };

    let hydrated = use_hydrated();

    view! {
        <div class="prose-page">
            <h1>{t!(i18n, sms_page_title)}</h1>

            // Error display
            {move || latest_error().map(|e| view! { <p class="alert">{e.to_string()}</p> })}

            // SMS report
            {move || {
                latest_report()
                    .map(|report| {
                        view! {
                            <div class="alert" data-testid="sms-report">
                                <p data-testid="sms-sent-confirmation">
                                    {t!(i18n, sms_sent_label)} <strong>{report.sent}</strong>
                                </p>
                                {if report.failed > 0 {
                                    view! {
                                        <p>
                                            {t!(i18n, sms_failed_label)}
                                            <strong>{report.failed}</strong>
                                        </p>
                                    }
                                        .into_any()
                                } else {
                                    ().into_any()
                                }}
                            </div>
                        }
                    })
            }}

            <section class="flex flex-col gap-3">
                // Story 5.3: Season-open — target all active users
                <div class="sms-trigger">
                    <h2>{t!(i18n, sms_season_open_section_title)}</h2>
                    <p>{t!(i18n, sms_season_open_target)}</p>
                    <leptos::form::ActionForm action=season_open_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-season-open-button"
                            disabled=move || !hydrated.get()
                        >
                            {t!(i18n, common_send_button)}
                        </button>
                    </leptos::form::ActionForm>
                </div>

                // Story 5.1: Assignment notification — target senders with notified_at IS NULL
                <div class="sms-trigger">
                    <h2>{t!(i18n, sms_assignment_section_title)}</h2>
                    <p>{t!(i18n, sms_assignment_target)}</p>
                    <leptos::form::ActionForm action=assignment_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-assignment-button"
                            disabled=move || !hydrated.get()
                        >
                            {t!(i18n, common_send_button)}
                        </button>
                    </leptos::form::ActionForm>
                </div>

                // Story 5.4: Pre-deadline nudge — target unconfirmed enrolled
                <div class="sms-trigger">
                    <h2>{t!(i18n, sms_confirm_nudge_section_title)}</h2>
                    <p>{t!(i18n, sms_confirm_nudge_target)}</p>
                    <leptos::form::ActionForm action=confirm_nudge_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-confirm-nudge-button"
                            disabled=move || !hydrated.get()
                        >
                            {t!(i18n, common_send_button)}
                        </button>
                    </leptos::form::ActionForm>
                </div>

                // Story 5.2: Receipt nudge — target recipients with no_response
                <div class="sms-trigger">
                    <h2>{t!(i18n, sms_receipt_nudge_section_title)}</h2>
                    <p>{t!(i18n, sms_receipt_nudge_target)}</p>
                    <leptos::form::ActionForm action=receipt_nudge_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-receipt-nudge-button"
                            disabled=move || !hydrated.get()
                        >
                            {t!(i18n, common_send_button)}
                        </button>
                    </leptos::form::ActionForm>
                </div>
            </section>
        </div>
    }
}

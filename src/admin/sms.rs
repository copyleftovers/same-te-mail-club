use crate::components::toast::use_toast;
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

/// Target counts for each SMS batch type.
///
/// Returned before the organizer sends — lets them know how many recipients
/// each SMS batch will reach without having to trigger the send to find out.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsCounts {
    /// Active participants who will receive the season-open notification.
    pub active_user_count: i64,
    /// Senders in the current assignment/delivery season not yet notified.
    pub unnotified_sender_count: i64,
    /// Enrolled users in the preparation phase who have not confirmed ready.
    pub unconfirmed_enrolled_count: i64,
    /// Recipients in the delivery phase who have not responded.
    pub no_response_count: i64,
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

/// Story 4.4: Target counts for each SMS batch type (admin only).
///
/// Fetched before the organizer sends so they know how many people will receive
/// each message. Counts are live: they update after each send.
///
/// # Errors
///
/// Returns `Err` if the caller is not admin or DB fails.
#[server]
pub async fn get_sms_counts() -> Result<SmsCounts, ServerFnError> {
    use crate::auth;

    let (pool, _user) = auth::require_admin().await?;

    let active_user_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!: i64" FROM users WHERE status = 'active' AND role = 'participant'"#,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    let unnotified_sender_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM assignments a
        JOIN seasons s ON a.season_id = s.id
        WHERE s.phase IN ('assignment', 'delivery') AND a.notified_at IS NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    let unconfirmed_enrolled_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM enrollments e
        JOIN seasons s ON e.season_id = s.id
        WHERE s.phase = 'preparation' AND e.confirmed_ready_at IS NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    let no_response_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!: i64"
        FROM assignments a
        JOIN seasons s ON a.season_id = s.id
        WHERE s.phase = 'delivery' AND a.receipt_status = 'no_response'
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;

    Ok(SmsCounts {
        active_user_count,
        unnotified_sender_count,
        unconfirmed_enrolled_count,
        no_response_count,
    })
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Admin SMS batch trigger page.
///
/// Each button fires one batch SMS job; result shown in `sms-report`.
/// Uses four separate actions so the result of each is tracked independently.
/// Target counts are displayed adjacent to each button and refresh after each send.
#[component]
pub fn SmsPage() -> impl IntoView {
    let i18n = use_i18n();
    let set_toast = use_toast();
    let season_open_action = ServerAction::<SendSeasonOpenSms>::new();
    let assignment_action = ServerAction::<SendAssignmentSms>::new();
    let confirm_nudge_action = ServerAction::<SendConfirmNudgeSms>::new();
    let receipt_nudge_action = ServerAction::<SendReceiptNudgeSms>::new();

    let hydrated = use_hydrated();

    // Counts refetch whenever any SMS action completes (version changes).
    // Sum all four action versions so any one of them triggers a refetch.
    let counts = Resource::new(
        move || {
            season_open_action.version().get()
                + assignment_action.version().get()
                + confirm_nudge_action.version().get()
                + receipt_nudge_action.version().get()
        },
        |_| get_sms_counts(),
    );

    // Toast feedback for successful SMS sends
    Effect::new(move |_| {
        if let Some(Ok(_)) = season_open_action.value().get() {
            set_toast.set(Some("SMS надіслано!".into()));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = assignment_action.value().get() {
            set_toast.set(Some("SMS надіслано!".into()));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = confirm_nudge_action.value().get() {
            set_toast.set(Some("SMS надіслано!".into()));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = receipt_nudge_action.value().get() {
            set_toast.set(Some("SMS надіслано!".into()));
        }
    });

    view! {
        <div class="prose-page">
            <h1>{t!(i18n, sms_page_title)}</h1>

            {render_sms_status(
                season_open_action,
                assignment_action,
                confirm_nudge_action,
                receipt_nudge_action,
                i18n,
            )}

            <Suspense fallback=|| ()>
                {move || {
                    let c = counts.get().and_then(Result::ok);
                    let active_user_count = c.as_ref().map_or(0, |x| x.active_user_count);
                    let unnotified_sender_count =
                        c.as_ref().map_or(0, |x| x.unnotified_sender_count);
                    let unconfirmed_enrolled_count =
                        c.as_ref().map_or(0, |x| x.unconfirmed_enrolled_count);
                    let no_response_count = c.as_ref().map_or(0, |x| x.no_response_count);
                    render_sms_triggers(
                        season_open_action,
                        assignment_action,
                        confirm_nudge_action,
                        receipt_nudge_action,
                        hydrated,
                        i18n,
                        active_user_count,
                        unnotified_sender_count,
                        unconfirmed_enrolled_count,
                        no_response_count,
                    )
                }}
            </Suspense>
        </div>
    }
}

/// Renders error display and SMS report from the most recent completed action.
fn render_sms_status(
    season_open_action: ServerAction<SendSeasonOpenSms>,
    assignment_action: ServerAction<SendAssignmentSms>,
    confirm_nudge_action: ServerAction<SendConfirmNudgeSms>,
    receipt_nudge_action: ServerAction<SendReceiptNudgeSms>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> impl IntoView {
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

    view! {
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
    }
}

/// Renders the four SMS trigger sections with live target counts.
// Four parallel sections (open, assign, confirm-nudge, receipt-nudge) × button + count span
// each. Extracting further would scatter a flat structure into 4+ tiny helpers with no gain.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_sms_triggers(
    season_open_action: ServerAction<SendSeasonOpenSms>,
    assignment_action: ServerAction<SendAssignmentSms>,
    confirm_nudge_action: ServerAction<SendConfirmNudgeSms>,
    receipt_nudge_action: ServerAction<SendReceiptNudgeSms>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
    active_user_count: i64,
    unnotified_sender_count: i64,
    unconfirmed_enrolled_count: i64,
    no_response_count: i64,
) -> impl IntoView {
    let season_open_pending = season_open_action.pending();
    let assignment_pending = assignment_action.pending();
    let confirm_nudge_pending = confirm_nudge_action.pending();
    let receipt_nudge_pending = receipt_nudge_action.pending();

    view! {
        <section class="flex flex-col gap-3">
            {render_sms_section(
                t!(i18n, sms_season_open_section_title),
                t!(i18n, sms_season_open_target),
                view! {
                    <leptos::form::ActionForm action=season_open_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-season-open-button"
                            disabled=move || season_open_pending.get() || !hydrated.get()
                        >
                            {move || if season_open_pending.get() {
                                "Надсилаю...".into_any()
                            } else {
                                t!(i18n, common_send_button).into_any()
                            }}
                        </button>
                    </leptos::form::ActionForm>
                },
                view! {
                    <span
                        class="text-sm text-[--color-text-muted]"
                        data-testid="sms-count-active-users"
                    >
                        {t!(i18n, sms_count_active_users, count = active_user_count)}
                    </span>
                },
            )}

            {render_sms_section(
                t!(i18n, sms_assignment_section_title),
                t!(i18n, sms_assignment_target),
                view! {
                    <leptos::form::ActionForm action=assignment_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-assignment-button"
                            disabled=move || assignment_pending.get() || !hydrated.get()
                        >
                            {move || if assignment_pending.get() {
                                "Надсилаю...".into_any()
                            } else {
                                t!(i18n, common_send_button).into_any()
                            }}
                        </button>
                    </leptos::form::ActionForm>
                },
                view! {
                    <span
                        class="text-sm text-[--color-text-muted]"
                        data-testid="sms-count-unnotified-senders"
                    >
                        {t!(i18n, sms_count_unnotified_senders, count = unnotified_sender_count)}
                    </span>
                },
            )}

            {render_sms_section(
                t!(i18n, sms_confirm_nudge_section_title),
                t!(i18n, sms_confirm_nudge_target),
                view! {
                    <leptos::form::ActionForm action=confirm_nudge_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-confirm-nudge-button"
                            disabled=move || confirm_nudge_pending.get() || !hydrated.get()
                        >
                            {move || if confirm_nudge_pending.get() {
                                "Надсилаю...".into_any()
                            } else {
                                t!(i18n, common_send_button).into_any()
                            }}
                        </button>
                    </leptos::form::ActionForm>
                },
                view! {
                    <span
                        class="text-sm text-[--color-text-muted]"
                        data-testid="sms-count-unconfirmed-enrolled"
                    >
                        {t!(
                            i18n,
                            sms_count_unconfirmed_enrolled,
                            count = unconfirmed_enrolled_count
                        )}
                    </span>
                },
            )}

            {render_sms_section(
                t!(i18n, sms_receipt_nudge_section_title),
                t!(i18n, sms_receipt_nudge_target),
                view! {
                    <leptos::form::ActionForm action=receipt_nudge_action>
                        <button
                            class="btn"
                            data-size="sm"
                            type="submit"
                            data-testid="send-receipt-nudge-button"
                            disabled=move || receipt_nudge_pending.get() || !hydrated.get()
                        >
                            {move || if receipt_nudge_pending.get() {
                                "Надсилаю...".into_any()
                            } else {
                                t!(i18n, common_send_button).into_any()
                            }}
                        </button>
                    </leptos::form::ActionForm>
                },
                view! {
                    <span
                        class="text-sm text-[--color-text-muted]"
                        data-testid="sms-count-no-response"
                    >
                        {t!(i18n, sms_count_no_response, count = no_response_count)}
                    </span>
                },
            )}
        </section>
    }
}

/// Renders a single SMS trigger section: heading, description, button, and count.
fn render_sms_section(
    title: impl IntoView,
    description: impl IntoView,
    button: impl IntoView,
    count_label: impl IntoView,
) -> impl IntoView {
    view! {
        <div class="sms-trigger">
            <h2>{title}</h2>
            <p>{description}</p>
            <div class="flex items-center gap-3">
                {button}
                {count_label}
            </div>
        </div>
    }
}

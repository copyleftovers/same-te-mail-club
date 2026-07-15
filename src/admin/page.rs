use crate::admin::assignments::{
    AssignmentLink, AssignmentPreview, CohortPreview, GenerateAssignments, SwapAssignment,
    get_assignment_preview,
};
use crate::admin::invite_codes::{
    DistributorOption, GenerateInviteCode, InviteCodeRow, RevokeInviteCode,
    list_distributor_options, list_invite_codes,
};
use crate::admin::participants::{DeactivateParticipant, ParticipantSummary, list_participants};
use crate::admin::season::{AdvanceSeason, CancelSeason, CreateSeason, LaunchSeason};
use crate::admin::sms::{
    SendAssignmentSms, SendConfirmNudgeSms, SendReceiptNudgeSms, SendSeasonOpenSms, SmsReport,
};
use crate::admin::state::{AdminSeason, AdminState, get_admin_state};
use crate::components::skeleton::SkeletonFallback;
use crate::components::stepper::PhaseStepper;
use crate::components::toast::use_toast;
use crate::error::{FIELD_DISCRIMINANT_SEPARATOR, strip_server_error_prefix};
use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, t_string, use_i18n};
use crate::types::InviteCodeStatus;
use leptos::prelude::*;

// ── Public page component ──────────────────────────────────────────────────────

/// Unified single-page admin interface (Stories 4.5 + 4.6).
///
/// Replaces the 4 separate admin pages (dashboard, season, assignments, sms)
/// with one phase-aware page. The season section morphs by phase; SMS buttons
/// appear only for the relevant phase. The participants section is always
/// visible below.
///
/// A single `get_admin_state()` resource drives the entire season section.
/// All mutations refetch the same resource via a tuple of all action versions.
#[allow(clippy::too_many_lines)]
#[component]
pub fn AdminPage() -> impl IntoView {
    let i18n = use_i18n();
    let set_toast = use_toast();
    let hydrated = use_hydrated();

    // ── Season mutation actions ────────────────────────────────────────────────
    let create_action = ServerAction::<CreateSeason>::new();
    let launch_action = ServerAction::<LaunchSeason>::new();
    let advance_action = ServerAction::<AdvanceSeason>::new();
    let cancel_action = ServerAction::<CancelSeason>::new();

    // ── Assignment mutation actions ────────────────────────────────────────────
    let generate_action = ServerAction::<GenerateAssignments>::new();
    let swap_action = ServerAction::<SwapAssignment>::new();

    // ── SMS mutation actions ───────────────────────────────────────────────────
    let season_open_action = ServerAction::<SendSeasonOpenSms>::new();
    let assignment_action = ServerAction::<SendAssignmentSms>::new();
    let confirm_nudge_action = ServerAction::<SendConfirmNudgeSms>::new();
    let receipt_nudge_action = ServerAction::<SendReceiptNudgeSms>::new();

    // ── Participant mutation actions ───────────────────────────────────────────
    let deactivate_action = ServerAction::<DeactivateParticipant>::new();

    // ── Invite code mutation actions ──────────────────────────────────────────
    let generate_invite_action = ServerAction::<GenerateInviteCode>::new();
    let revoke_invite_action = ServerAction::<RevokeInviteCode>::new();

    // ── Unified state resource ─────────────────────────────────────────────────
    // Any completed mutation triggers a full state refetch.
    let admin_state = Resource::new(
        move || {
            (
                create_action.version().get(),
                launch_action.version().get(),
                advance_action.version().get(),
                cancel_action.version().get(),
                generate_action.version().get(),
                swap_action.version().get(),
                season_open_action.version().get(),
                assignment_action.version().get(),
                confirm_nudge_action.version().get(),
                receipt_nudge_action.version().get(),
                deactivate_action.version().get(),
            )
        },
        |_| get_admin_state(),
    );

    // ── Separate assignment preview resource ──────────────────────────────────
    // Needed for cycle visualization and swap form (requires full chain data).
    let preview = Resource::new(
        move || (generate_action.version().get(), swap_action.version().get()),
        |_| get_assignment_preview(),
    );

    // ── Participant list resource ─────────────────────────────────────────────
    let participants = Resource::new(
        move || deactivate_action.version().get(),
        |_| list_participants(),
    );

    // ── Invite codes resource ─────────────────────────────────────────────────
    // Refetches after generate or revoke completes.
    let invite_codes = Resource::new(
        move || {
            (
                generate_invite_action.version().get(),
                revoke_invite_action.version().get(),
            )
        },
        |_| list_invite_codes(),
    );

    // ── Distributor options resource ──────────────────────────────────────────
    // Refetches after generate (in case a new participant registered via code).
    let distributor_options = Resource::new(
        move || generate_invite_action.version().get(),
        |_| list_distributor_options(),
    );

    // ── Toast feedback for successful actions ─────────────────────────────────
    Effect::new(move |_| {
        if let Some(Ok(())) = create_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_season_created_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = launch_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_season_launched_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = advance_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_season_advanced_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = cancel_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_season_cancelled_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(_)) = generate_action.value().get() {
            set_toast.set(Some(
                t_string!(i18n, admin_assignments_generated_toast).into(),
            ));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(_)) = season_open_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_sms_sent_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(_)) = assignment_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_sms_sent_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(_)) = confirm_nudge_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_sms_sent_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(_)) = receipt_nudge_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_sms_sent_toast).into()));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = deactivate_action.value().get() {
            set_toast.set(Some(
                t_string!(i18n, admin_participant_deactivated_toast).into(),
            ));
        }
    });
    Effect::new(move |_| {
        if generate_invite_action
            .value()
            .get()
            .is_some_and(|r| r.is_ok())
        {
            set_toast.set(Some(
                t_string!(i18n, admin_invite_codes_generated_toast).into(),
            ));
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = revoke_invite_action.value().get() {
            set_toast.set(Some(
                t_string!(i18n, admin_invite_codes_revoked_toast).into(),
            ));
        }
    });

    // ── Collect errors from all actions ───────────────────────────────────────
    // create_action's field-discriminated errors (signup/confirm deadline) ALSO
    // render locally next to their input (see render_create_form); this banner
    // keeps showing the stripped message too (existing E2E contract on
    // `action-error` for create-season validation), field-less errors (e.g.
    // "active season already exists") have no other home.
    //
    // generate_invite_action has exactly one field (distributor); every error it
    // can produce belongs to that field's local `.field-error` (see
    // InviteCodesSection), never this banner.
    let action_error = move || {
        create_action
            .value()
            .get()
            .and_then(Result::err)
            .or_else(|| launch_action.value().get().and_then(Result::err))
            .or_else(|| advance_action.value().get().and_then(Result::err))
            .or_else(|| cancel_action.value().get().and_then(Result::err))
            .or_else(|| generate_action.value().get().and_then(Result::err))
            .or_else(|| swap_action.value().get().and_then(Result::err))
            .or_else(|| season_open_action.value().get().and_then(Result::err))
            .or_else(|| assignment_action.value().get().and_then(Result::err))
            .or_else(|| confirm_nudge_action.value().get().and_then(Result::err))
            .or_else(|| receipt_nudge_action.value().get().and_then(Result::err))
            .or_else(|| deactivate_action.value().get().and_then(Result::err))
            .or_else(|| revoke_invite_action.value().get().and_then(Result::err))
    };

    view! {
        <div class="prose-page" data-testid="dashboard-content">
            // FU-21: brand-anchor page title — admin previously had no h1, so the
            // heading scale started at h2 (section) with nothing above it.
            <h1>{t!(i18n, admin_page_title)}</h1>
            <div id="action-error" role="alert" aria-live="assertive" data-testid="action-error">
                {move || action_error().map(|e| {
                    let stripped = strip_server_error_prefix(&e);
                    // If the error carries a field discriminant (create-season
                    // validation), show only the user-facing message portion.
                    let (_field, display_msg) = parse_create_season_field_error(&stripped);
                    view! { <p class="alert">{display_msg.to_owned()}</p> }
                })}
            </div>

            // ── Season section ─────────────────────────────────────────────────
            <section class="admin-section" data-testid="season-section">
                <h2>{t!(i18n, season_page_title)}</h2>
                <Suspense fallback=move || view! { <SkeletonFallback /> }>
                    {move || {
                        admin_state
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="alert">{strip_server_error_prefix(&e)}</p> }.into_any(),
                                Ok(ref state) => render_season_section(
                                    state,
                                    create_action,
                                    launch_action,
                                    advance_action,
                                    cancel_action,
                                    season_open_action,
                                    confirm_nudge_action,
                                    assignment_action,
                                    receipt_nudge_action,
                                    hydrated,
                                    i18n,
                                ),
                            })
                    }}
                </Suspense>
            </section>

            // ── Assignments section — only shown when a preview exists ─────────
            <Suspense fallback=|| ()>
                {move || {
                    preview.get().map(|result| match result {
                        Ok(Some(ref p)) => {
                            render_assignment_section(p, generate_action, swap_action, hydrated, i18n)
                        }
                        _ => ().into_any(),
                    })
                }}
            </Suspense>

            // ── Participants (peer top-level sections, no group wrapper) ───────
            // FU-21: the "Учасники" group h2 wrapped these two cards, whose own
            // titles are ALSO h2 — a same-level collision (FU-18's root cause).
            // Dropped: invite-codes and participant-list stand as peer
            // .admin-section cards directly under the page h1, same as
            // season-section/assignment-section above.
            <InviteCodesSection
                generate_invite_action=generate_invite_action
                revoke_invite_action=revoke_invite_action
                invite_codes=invite_codes
                distributor_options=distributor_options
                hydrated=hydrated
            />
            <section class="admin-section" data-testid="participant-list-section">
                <h2>{t!(i18n, participants_list_title)}</h2>
                <ParticipantListSection
                    participants=participants
                    deactivate_action=deactivate_action
                />
            </section>
        </div>
    }
}

// ── Season section rendering ───────────────────────────────────────────────────

/// Render the season section based on the current `AdminState`.
///
/// Dispatches to phase-specific sub-renders. When no season exists, shows the
/// create form. Otherwise shows the season detail and phase-appropriate controls.
#[allow(clippy::too_many_arguments)]
fn render_season_section(
    state: &AdminState,
    create_action: ServerAction<CreateSeason>,
    launch_action: ServerAction<LaunchSeason>,
    advance_action: ServerAction<AdvanceSeason>,
    cancel_action: ServerAction<CancelSeason>,
    season_open_action: ServerAction<SendSeasonOpenSms>,
    confirm_nudge_action: ServerAction<SendConfirmNudgeSms>,
    assignment_action: ServerAction<SendAssignmentSms>,
    receipt_nudge_action: ServerAction<SendReceiptNudgeSms>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    match &state.season {
        None => render_create_form(create_action, hydrated, i18n, CreateFormContext::Standalone),
        Some(season) => render_active_season(
            season,
            state.participant_count,
            create_action,
            launch_action,
            advance_action,
            cancel_action,
            season_open_action,
            confirm_nudge_action,
            assignment_action,
            receipt_nudge_action,
            hydrated,
            i18n,
        ),
    }
}

/// Which create-season field the server rejected.
#[derive(Clone, PartialEq)]
enum CreateSeasonRejectedField {
    SignupDeadline,
    ConfirmDeadline,
}

/// Split a stripped server error into `(field, display_message)`.
///
/// The server encodes deadline validation errors as `"field_key\u{1f}message"`.
/// Returns `(Some(field), message)` for field-keyed errors; `(None, full_string)`
/// for infra/auth/DB errors with no separator — those show on the banner only.
fn parse_create_season_field_error(stripped: &str) -> (Option<CreateSeasonRejectedField>, &str) {
    if let Some((key, msg)) = stripped.split_once(FIELD_DISCRIMINANT_SEPARATOR) {
        let field = match key {
            "signup_deadline" => Some(CreateSeasonRejectedField::SignupDeadline),
            "confirm_deadline" => Some(CreateSeasonRejectedField::ConfirmDeadline),
            _ => None,
        };
        (field, msg)
    } else {
        (None, stripped)
    }
}

/// Where the create-season form is mounted — controls whether the "no active
/// season" heading above it is shown.
///
/// RV-13: when the form is embedded below an already-terminal season (a
/// ЗАВЕРШЕНО/СКАСОВАНО badge is already visible above it), restating "no
/// active season" is redundant with that badge. When the form is the page's
/// only content (no season row exists yet), the same line is the primary,
/// meaningful heading.
#[derive(Clone, Copy, PartialEq)]
enum CreateFormContext {
    Standalone,
    AfterTerminalSeason,
}

/// Render the create-season form.
// The `view!` macro's HTML attribute verbosity inflates line count beyond what
// reflects logic complexity; extracting sub-components here would be YAGNI.
#[allow(clippy::too_many_lines)]
fn render_create_form(
    create_action: ServerAction<CreateSeason>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
    context: CreateFormContext,
) -> AnyView {
    let pending = create_action.pending();
    // Per-field flags drive `aria-invalid` (the red border on the rejected
    // input) only. RV-04: the error MESSAGE itself is shown once, in the
    // page-wide `#action-error` banner below — a local text echo here would
    // duplicate that same fact. `aria-describedby="action-error"` links each
    // input to that single message, matching the pattern already used by the
    // swap form's `sender-a`/`sender-b` selects.
    let (signup_error, set_signup_error) = signal(false);
    let (confirm_error, set_confirm_error) = signal(false);

    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            match result {
                Ok(()) => {
                    set_signup_error.set(false);
                    set_confirm_error.set(false);
                }
                Err(e) => {
                    let stripped = strip_server_error_prefix(&e);
                    let (field, _msg) = parse_create_season_field_error(&stripped);
                    match field {
                        Some(CreateSeasonRejectedField::SignupDeadline) => {
                            set_signup_error.set(true);
                            set_confirm_error.set(false);
                        }
                        Some(CreateSeasonRejectedField::ConfirmDeadline) => {
                            set_signup_error.set(false);
                            set_confirm_error.set(true);
                        }
                        None => {
                            // Infra/auth/active-exists errors: no field border,
                            // shown on the page-wide banner instead.
                            set_signup_error.set(false);
                            set_confirm_error.set(false);
                        }
                    }
                }
            }
        }
    });

    view! {
        <div data-testid="create-season-form">
            {(context == CreateFormContext::Standalone).then(|| view! {
                <p class="text-[length:var(--text-secondary)] text-(--color-text-muted) mb-(--density-space-sm)">
                    {t!(i18n, home_no_season_heading)}
                </p>
            })}
            <section>
                <h3 class="overline-label">{t!(i18n, season_create_form_title)}</h3>
                <leptos::form::ActionForm action=create_action>
                    <div class="field">
                        <label class="field-label" for="signup-deadline">
                            {t!(i18n, season_signup_deadline_label)}
                        </label>
                        <input
                            class="field-input"
                            id="signup-deadline"
                            type="datetime-local"
                            name="signup_deadline"
                            required=true
                            data-testid="signup-deadline-input"
                            aria-describedby="action-error"
                            aria-invalid=move || signup_error.get().then_some("true")
                        />
                    </div>
                    <div class="field">
                        <label class="field-label" for="confirm-deadline">
                            {t!(i18n, season_confirm_deadline_label)}
                        </label>
                        <input
                            class="field-input"
                            id="confirm-deadline"
                            type="datetime-local"
                            name="confirm_deadline"
                            required=true
                            data-testid="confirm-deadline-input"
                            aria-describedby="action-error"
                            aria-invalid=move || confirm_error.get().then_some("true")
                        />
                    </div>
                    <div class="field">
                        <label class="field-label" for="theme">
                            {t!(i18n, season_theme_label)}
                        </label>
                        <input
                            class="field-input"
                            id="theme"
                            type="text"
                            name="theme"
                            maxlength="100"
                            placeholder=move || t_string!(i18n, season_theme_placeholder)
                            data-testid="theme-input"
                        />
                    </div>
                    <button
                        class="btn"
                        type="submit"
                        data-testid="create-season-button"
                        disabled=move || pending.get() || !hydrated.get()
                        attr:aria-busy=move || pending.get().then_some("true")
                    >
                        {move || if pending.get() {
                            t!(i18n, admin_season_creating_loading).into_any()
                        } else {
                            t!(i18n, season_create_button).into_any()
                        }}
                    </button>
                </leptos::form::ActionForm>
            </section>
        </div>
    }
    .into_any()
}

/// Render an active season — controls differ by phase and launched state.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_active_season(
    season: &AdminSeason,
    participant_count: i64,
    create_action: ServerAction<CreateSeason>,
    launch_action: ServerAction<LaunchSeason>,
    advance_action: ServerAction<AdvanceSeason>,
    cancel_action: ServerAction<CancelSeason>,
    season_open_action: ServerAction<SendSeasonOpenSms>,
    confirm_nudge_action: ServerAction<SendConfirmNudgeSms>,
    assignment_action: ServerAction<SendAssignmentSms>,
    receipt_nudge_action: ServerAction<SendReceiptNudgeSms>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    let launch_pending = launch_action.pending();
    let advance_pending = advance_action.pending();
    let cancel_pending = cancel_action.pending();

    let can_advance = season.phase.can_advance();
    let launched = season.launched;
    let is_terminal = season.phase.is_terminal();

    // Advance from Assignment → Delivery requires assignments to have been released.
    let advance_blocked =
        season.phase == crate::types::Phase::Assignment && !season.assignments_released;

    let (confirming, set_confirming) = signal(false);

    // Reset confirmation state when cancel action succeeds.
    Effect::new(move |_| {
        if let Some(Ok(())) = cancel_action.value().get() {
            set_confirming.set(false);
        }
    });

    // Timestamps are pre-formatted strings in AdminSeason — no SSR/WASM branching needed.
    let signup_deadline_str = season.signup_deadline.clone();
    let confirm_deadline_str = season.confirm_deadline.clone();

    // Clone data needed in closures.
    let theme = season.theme.clone();
    let enrolled_count = season.enrolled_count;
    let confirmed_count = season.confirmed_count;
    let not_received_count = season.not_received_count;
    let season_open_target_count = season.season_open_target_count;
    let unnotified_sender_count = season.unnotified_sender_count;
    let unconfirmed_enrolled_count = season.unconfirmed_enrolled_count;
    let no_response_count = season.no_response_count;

    let phase = season.phase;

    view! {
        <div>
            <PhaseStepper current_phase=phase />

            // Season summary: theme + deadlines + counts
            <dl data-testid="season-summary">
                {theme.as_ref().map(|theme_val| view! {
                    <dt>{t!(i18n, season_theme_display_label)}</dt>
                    <dd data-testid="season-theme">{theme_val.clone()}</dd>
                })}
                <dt>{t!(i18n, season_signup_deadline_display)}</dt>
                <dd data-testid="season-deadline">{signup_deadline_str.clone()}</dd>
                <dt>{t!(i18n, season_confirm_deadline_display)}</dt>
                <dd>{confirm_deadline_str.clone()}</dd>
                {if is_terminal {
                    ().into_any()
                } else if launched {
                    view! {
                        <dt>{t!(i18n, season_enrolled_label)}</dt>
                        <dd>{enrolled_count.to_string()}</dd>
                        <dt>{t!(i18n, season_confirmed_label)}</dt>
                        <dd data-testid="confirmed-count">{confirmed_count.to_string()}</dd>
                    }.into_any()
                } else {
                    // Pre-launch: enrolled/confirmed are structurally 0 (enrollment
                    // requires launched_at IS NOT NULL). Show the participant pool
                    // size as the single meaningful metric — as a <dl> row, not a
                    // stray <p>. Reuses the existing bare-label key (uk.json:78).
                    view! {
                        <dt>{t!(i18n, admin_pre_launch_participant_count)}</dt>
                        <dd data-testid="pre-launch-participant-count">{participant_count.to_string()}</dd>
                    }.into_any()
                }}
            </dl>

            // Terminal-state badge — inline status indicator for Cancelled/Complete
            {if is_terminal {
                let (status, label) = if phase == crate::types::Phase::Cancelled {
                    ("inactive", t_string!(i18n, season_phase_cancelled))
                } else {
                    ("confirmed", t_string!(i18n, season_phase_complete))
                };
                view! {
                    <span class="badge" data-status=status data-testid="season-terminal-badge">
                        {label}
                    </span>
                }.into_any()
            } else {
                ().into_any()
            }}

            // Not-received alert (only in delivery/complete with non-zero count).
            // Bare `<p class="alert">` — the canonical alert shape shared with the
            // page-wide banner and the distributor/state error alerts.
            {if not_received_count > 0 {
                view! {
                    <p class="alert" data-testid="not-received-alert">
                        {t!(i18n, admin_not_received_label)}
                        {not_received_count}
                    </p>
                }.into_any()
            } else {
                ().into_any()
            }}

            // Phase-conditional SMS section (Story 4.6)
            {render_phase_sms(
                phase,
                launched,
                season_open_target_count,
                unnotified_sender_count,
                unconfirmed_enrolled_count,
                no_response_count,
                season_open_action,
                confirm_nudge_action,
                assignment_action,
                receipt_nudge_action,
                hydrated,
                i18n,
            )}

            // Action buttons: launch, advance, cancel
            <div class="flex flex-wrap items-start gap-(--density-space-sm) mt-(--density-space-md)" data-testid="season-action-buttons">
                // Launch — only when not yet launched and not terminal
                {if !launched && !is_terminal {
                    view! {
                        <leptos::form::ActionForm action=launch_action>
                            <button
                                class="btn"
                                type="submit"
                                data-testid="launch-button"
                                disabled=move || launch_pending.get() || !hydrated.get()
                                attr:aria-busy=move || launch_pending.get().then_some("true")
                            >
                                {move || if launch_pending.get() {
                                    t!(i18n, admin_season_launching_loading).into_any()
                                } else {
                                    t!(i18n, season_launch_button).into_any()
                                }}
                            </button>
                        </leptos::form::ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}

                // Advance — only when launched and phase can advance
                {if launched && can_advance {
                    view! {
                        <div class="flex flex-col gap-(--density-space-sm)">
                            <leptos::form::ActionForm action=advance_action>
                                // Primary (default) variant: advancing the phase is
                                // the affirmative main action of the launched season —
                                // it must be the loudest button, above the destructive
                                // "Скасувати" (S3). Was recessive `secondary`.
                                <button
                                    class="btn"
                                    type="submit"
                                    data-testid="advance-button"
                                    disabled=move || {
                                        advance_pending.get() || !hydrated.get() || advance_blocked
                                    }
                                    attr:aria-busy=move || advance_pending.get().then_some("true")
                                >
                                    {move || if advance_pending.get() {
                                        t!(i18n, admin_season_advancing_loading).into_any()
                                    } else {
                                        t!(i18n, season_advance_button).into_any()
                                    }}
                                </button>
                            </leptos::form::ActionForm>
                            {if advance_blocked {
                                view! {
                                    <p
                                        class="text-[length:var(--text-secondary)] text-(--color-text-muted)"
                                        data-testid="advance-blocked-hint"
                                    >
                                        {t!(i18n, season_advance_blocked_hint)}
                                    </p>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }}

                // Cancel — two-step confirmation, available any time before terminal
                {if is_terminal {
                    ().into_any()
                } else {
                    view! {
                        <Show
                            when=move || !confirming.get()
                            fallback=move || {
                                view! {
                                    <div data-testid="cancel-confirmation">
                                        <p>{t!(i18n, season_cancel_confirm_prompt)}</p>
                                        // Adjacent Так/Ні buttons use the .btn-group primitive
                                        // (flex + wrap + gap) rather than ad-hoc flex utilities (S7).
                                        <div class="btn-group mt-(--density-space-sm)">
                                            <leptos::form::ActionForm action=cancel_action>
                                                <button
                                                    class="btn"
                                                    data-variant="destructive"
                                                    type="submit"
                                                    data-testid="cancel-confirm-button"
                                                    disabled=move || cancel_pending.get() || !hydrated.get()
                                                    attr:aria-busy=move || cancel_pending.get().then_some("true")
                                                >
                                                    {move || if cancel_pending.get() {
                                                        t!(i18n, admin_season_cancelling_loading).into_any()
                                                    } else {
                                                        t!(i18n, season_cancel_confirm_yes).into_any()
                                                    }}
                                                </button>
                                            </leptos::form::ActionForm>
                                            <button
                                                class="btn"
                                                data-variant="secondary"
                                                type="button"
                                                data-testid="cancel-back-button"
                                                on:click=move |_| set_confirming.set(false)
                                            >
                                                {t!(i18n, season_cancel_confirm_no)}
                                            </button>
                                        </div>
                                    </div>
                                }
                            }
                        >
                            <button
                                class="btn"
                                data-variant="secondary"
                                type="button"
                                data-testid="cancel-button"
                                disabled=move || !hydrated.get()
                                on:click=move |_| set_confirming.set(true)
                            >
                                {t!(i18n, season_cancel_button)}
                            </button>
                        </Show>
                    }.into_any()
                }}
            </div>

            // Terminal state: offer a fresh season below the concluded-season
            // summary. A top border + generous spacing separate the read-only
            // record above from the create-next-season form, so the summary's
            // deadline facts and the form's editable deadline inputs no longer
            // read as one braided block.
            {if is_terminal {
                view! {
                    <div class="mt-(--density-space-lg) pt-(--density-space-lg) border-t border-(--color-border)">
                        {render_create_form(
                            create_action,
                            hydrated,
                            i18n,
                            CreateFormContext::AfterTerminalSeason,
                        )}
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
    .into_any()
}

// ── Phase-conditional SMS section (Story 4.6) ──────────────────────────────────

/// Render SMS buttons only for the relevant phase.
///
/// - Enrollment: season-open SMS only
/// - Preparation: confirm nudge only
/// - Delivery: assignment SMS + receipt nudge
/// - All other phases: nothing
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_phase_sms(
    phase: crate::types::Phase,
    launched: bool,
    season_open_target_count: i64,
    unnotified_sender_count: i64,
    unconfirmed_enrolled_count: i64,
    no_response_count: i64,
    season_open_action: ServerAction<SendSeasonOpenSms>,
    confirm_nudge_action: ServerAction<SendConfirmNudgeSms>,
    assignment_action: ServerAction<SendAssignmentSms>,
    receipt_nudge_action: ServerAction<SendReceiptNudgeSms>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    use crate::types::Phase;

    if !launched {
        return ().into_any();
    }

    match phase {
        Phase::Enrollment => {
            let season_open_pending = season_open_action.pending();
            view! {
                <div class="flex flex-col gap-(--density-space-sm) mt-(--density-space-md)" data-testid="sms-section-enrollment">
                    <div class="sms-trigger">
                        <h3>{t!(i18n, sms_season_open_section_title)}</h3>
                        <p>{t!(i18n, sms_season_open_target)}</p>
                        <div class="sms-trigger-action">
                            <leptos::form::ActionForm action=season_open_action>
                                <button
                                    class="btn"
                                    data-variant="secondary"
                                    data-size="sm"
                                    type="submit"
                                    data-testid="send-season-open-button"
                                    disabled=move || season_open_pending.get() || !hydrated.get()
                                    attr:aria-busy=move || season_open_pending.get().then_some("true")
                                >
                                    {move || if season_open_pending.get() {
                                        t!(i18n, admin_sms_sending_loading).into_any()
                                    } else {
                                        t!(i18n, sms_send_season_open_button).into_any()
                                    }}
                                </button>
                            </leptos::form::ActionForm>
                            <span
                                class="text-[length:var(--text-secondary)] text-(--color-text-muted)"
                                data-testid="sms-count-active-users"
                            >
                                {t!(i18n, sms_count_active_users, count = season_open_target_count)}
                            </span>
                        </div>
                        {render_sms_report_inline(move || season_open_action.value().get().and_then(Result::ok), i18n)}
                    </div>
                </div>
            }
            .into_any()
        }
        Phase::Preparation => {
            let confirm_nudge_pending = confirm_nudge_action.pending();
            view! {
                <div class="flex flex-col gap-(--density-space-sm) mt-(--density-space-md)" data-testid="sms-section-preparation">
                    <div class="sms-trigger">
                        <h3>{t!(i18n, sms_confirm_nudge_section_title)}</h3>
                        <p>{t!(i18n, sms_confirm_nudge_target)}</p>
                        <div class="sms-trigger-action">
                            <leptos::form::ActionForm action=confirm_nudge_action>
                                <button
                                    class="btn"
                                    data-variant="secondary"
                                    data-size="sm"
                                    type="submit"
                                    data-testid="send-confirm-nudge-button"
                                    disabled=move || confirm_nudge_pending.get() || !hydrated.get()
                                    attr:aria-busy=move || confirm_nudge_pending.get().then_some("true")
                                >
                                    {move || if confirm_nudge_pending.get() {
                                        t!(i18n, admin_sms_sending_loading).into_any()
                                    } else {
                                        t!(i18n, sms_send_confirm_nudge_button).into_any()
                                    }}
                                </button>
                            </leptos::form::ActionForm>
                            <span
                                class="text-[length:var(--text-secondary)] text-(--color-text-muted)"
                                data-testid="sms-count-unconfirmed-enrolled"
                            >
                                {t!(
                                    i18n,
                                    sms_count_unconfirmed_enrolled,
                                    count = unconfirmed_enrolled_count
                                )}
                            </span>
                        </div>
                        {render_sms_report_inline(move || confirm_nudge_action.value().get().and_then(Result::ok), i18n)}
                    </div>
                </div>
            }
            .into_any()
        }
        Phase::Delivery => {
            let assignment_pending = assignment_action.pending();
            let receipt_nudge_pending = receipt_nudge_action.pending();
            view! {
                <div class="flex flex-col gap-(--density-space-sm) mt-(--density-space-md)" data-testid="sms-section-delivery">
                    <div class="sms-trigger">
                        <h3>{t!(i18n, sms_assignment_section_title)}</h3>
                        <p>{t!(i18n, sms_assignment_target)}</p>
                        <div class="sms-trigger-action">
                            <leptos::form::ActionForm action=assignment_action>
                                <button
                                    class="btn"
                                    data-variant="secondary"
                                    data-size="sm"
                                    type="submit"
                                    data-testid="send-assignment-button"
                                    disabled=move || assignment_pending.get() || !hydrated.get()
                                    attr:aria-busy=move || assignment_pending.get().then_some("true")
                                >
                                    {move || if assignment_pending.get() {
                                        t!(i18n, admin_sms_sending_loading).into_any()
                                    } else {
                                        t!(i18n, sms_send_assignment_button).into_any()
                                    }}
                                </button>
                            </leptos::form::ActionForm>
                            <span
                                class="text-[length:var(--text-secondary)] text-(--color-text-muted)"
                                data-testid="sms-count-unnotified-senders"
                            >
                                {t!(
                                    i18n,
                                    sms_count_unnotified_senders,
                                    count = unnotified_sender_count
                                )}
                            </span>
                        </div>
                        {render_sms_report_inline(move || assignment_action.value().get().and_then(Result::ok), i18n)}
                    </div>
                    <div class="sms-trigger">
                        <h3>{t!(i18n, sms_receipt_nudge_section_title)}</h3>
                        <p>{t!(i18n, sms_receipt_nudge_target)}</p>
                        <div class="sms-trigger-action">
                            <leptos::form::ActionForm action=receipt_nudge_action>
                                <button
                                    class="btn"
                                    data-variant="secondary"
                                    data-size="sm"
                                    type="submit"
                                    data-testid="send-receipt-nudge-button"
                                    disabled=move || receipt_nudge_pending.get() || !hydrated.get()
                                    attr:aria-busy=move || receipt_nudge_pending.get().then_some("true")
                                >
                                    {move || if receipt_nudge_pending.get() {
                                        t!(i18n, admin_sms_sending_loading).into_any()
                                    } else {
                                        t!(i18n, sms_send_receipt_nudge_button).into_any()
                                    }}
                                </button>
                            </leptos::form::ActionForm>
                            <span
                                class="text-[length:var(--text-secondary)] text-(--color-text-muted)"
                                data-testid="sms-count-no-response"
                            >
                                {t!(i18n, sms_count_no_response, count = no_response_count)}
                            </span>
                        </div>
                        {render_sms_report_inline(move || receipt_nudge_action.value().get().and_then(Result::ok), i18n)}
                    </div>
                </div>
            }
            .into_any()
        }
        Phase::Assignment | Phase::Complete | Phase::Cancelled => ().into_any(),
    }
}

// ── SMS report rendering ───────────────────────────────────────────────────────

/// Render an inline SMS report with sent/failed badge counts.
///
/// Always shows both counts (including zeros) using `.badge[data-status]` —
/// `active` for sent (green = success), `error`/`inactive` for failed.
/// Placed inside the trigger card that fired the action.
fn render_sms_report_inline(
    report_signal: impl Fn() -> Option<SmsReport> + Send + Sync + 'static,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> impl IntoView {
    view! {
        {move || {
            report_signal().map(|report| {
                let failed_status = if report.failed > 0 { "error" } else { "inactive" };
                view! {
                    <div class="sms-report-result" data-testid="sms-report">
                        <span class="badge" data-status="active" data-testid="sms-sent-confirmation">
                            {t!(i18n, sms_sent_label)} {report.sent}
                        </span>
                        <span class="badge" data-status=failed_status>
                            {t!(i18n, sms_failed_label)} {report.failed}
                        </span>
                    </div>
                }
            })
        }}
    }
}

// ── Assignment section rendering ───────────────────────────────────────────────

/// Render the assignments sub-section within the season section.
///
/// Only shown when the season is in Assignment phase (or later) and a preview
/// is available. Generate button appears only in Assignment phase.
fn render_assignment_section(
    p: &AssignmentPreview,
    generate_action: ServerAction<GenerateAssignments>,
    swap_action: ServerAction<SwapAssignment>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    use crate::types::Phase;

    let is_assignment_phase = p.phase == Phase::Assignment;
    let has_assignments = !p.cohorts.is_empty() && !p.cohorts.iter().all(|c| c.chain.is_empty());

    // Only render this section during Assignment phase or later (when assignments exist)
    if !is_assignment_phase && !has_assignments {
        return ().into_any();
    }

    let season_id = p.season_id.clone();
    let generate_pending = generate_action.pending();

    let all_links: Vec<AssignmentLink> = p
        .cohorts
        .iter()
        .flat_map(|c| c.chain.iter().cloned())
        .collect();

    let cohorts_for_viz = p.cohorts.clone();

    view! {
        <section class="admin-section" data-testid="assignment-section">
            <h2>{t!(i18n, assignments_page_title)}</h2>

            // Generate button (only in assignment phase)
            {if is_assignment_phase {
                view! {
                    <leptos::form::ActionForm action=generate_action>
                        // Secondary variant (FU-20): card-local routine action, not
                        // the page's phase-transition control — leaves exactly one
                        // orange primary ("Далі") per admin phase.
                        <button
                            class="btn"
                            data-variant="secondary"
                            type="submit"
                            data-testid="generate-button"
                            disabled=move || generate_pending.get() || !hydrated.get()
                            attr:aria-busy=move || generate_pending.get().then_some("true")
                        >
                            {move || if generate_pending.get() {
                                t!(i18n, admin_assignments_generating_loading).into_any()
                            } else if has_assignments {
                                t!(i18n, assignments_regenerate_button).into_any()
                            } else {
                                t!(i18n, assignments_generate_button).into_any()
                            }}
                        </button>
                    </leptos::form::ActionForm>
                }.into_any()
            } else {
                view! {
                    <p data-testid="released-status">{t!(i18n, assignments_released_note)}</p>
                }.into_any()
            }}

            // Cycle visualization
            {if has_assignments {
                render_cycle_visualization(&cohorts_for_viz, i18n).into_any()
            } else {
                ().into_any()
            }}

            // Swap UI — only in assignment phase with assignments
            {if has_assignments && is_assignment_phase {
                view! {
                    <SwapFormSection
                        swap_action=swap_action
                        season_id=season_id
                        hydrated=hydrated
                        i18n=i18n
                        links=all_links
                    />
                }.into_any()
            } else {
                ().into_any()
            }}
        </section>
    }
    .into_any()
}

/// Render the cycle visualization for all cohorts.
fn render_cycle_visualization(
    cohorts: &[CohortPreview],
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> impl IntoView {
    let cohorts_view = cohorts
        .iter()
        .enumerate()
        .map(|(idx, cohort)| render_cycle_ring(&cohort.chain, idx + 1, cohort.score))
        .collect_view();

    view! {
        <div>
            <h3>{t!(i18n, assignments_cycles_label)}</h3>
            {cohorts_view}
        </div>
    }
}

/// Compute circle node positions for SVG ring.
#[allow(clippy::cast_precision_loss)]
fn compute_circle_positions(
    n: usize,
    radius: f64,
    center_x: f64,
    center_y: f64,
) -> Vec<(f64, f64)> {
    use std::f64::consts::{PI, TAU};
    (0..n)
        .map(|i| {
            let angle = (i as f64 / n as f64) * TAU - (PI / 2.0);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            (x, y)
        })
        .collect()
}

/// Smallest cohort the assignment algorithm ever produces (see `split_cohorts`
/// in `assignment.rs`, which groups participants into cohorts of 3-15).
const MIN_COHORT_SIZE: usize = 3;
/// Cohort size at which the ring reaches `MAX_RING_RADIUS`. Matches the 12-node
/// capture harness (`end2end/tests/fixtures/cohort-seed.sql`) — the only cohort
/// size this label layout (two-line split + quadrant-aware anchors, below) has
/// been verified collision-free against. Cohorts larger than this, up to the
/// algorithm's ceiling of 15, reuse the same radius rather than growing further
/// into untested territory.
const MAX_VALIDATED_COHORT_SIZE: usize = 12;
/// Ring radius at the smallest possible cohort. A fixed worst-case radius would
/// strand a 3-node cycle in mostly-empty canvas, so radius grows from here
/// toward `MAX_RING_RADIUS` as cohort size approaches `MAX_VALIDATED_COHORT_SIZE`.
const MIN_RING_RADIUS: f64 = 90.0;
const MAX_RING_RADIUS: f64 = 190.0;
/// Node dot radius at the smallest possible cohort. Fewer nodes need less
/// circumferential room, so the dot can afford to shrink alongside the ring —
/// see `MAX_NODE_RADIUS` for the validated large-cohort size.
const MIN_NODE_RADIUS: f64 = 14.0;
/// Node dot radius at `MAX_VALIDATED_COHORT_SIZE` — the previously-validated,
/// collision-free size for a densely-packed 12-node ring.
const MAX_NODE_RADIUS: f64 = 20.0;
/// Gap from node edge to label anchor, measured radially outward.
const LABEL_RADIAL_GAP: f64 = 14.0;
/// Clearance from node edge to arrow tip/tail so arrows don't overlap nodes.
const ARROW_CLEARANCE: f64 = 5.0;
/// Nodes within this many degrees of the vertical axis use middle text-anchor.
const VERTICAL_AXIS_TOLERANCE_DEGREES: f64 = 20.0;
/// Font size (SVG user units) for every node label. Held constant across
/// cohort sizes — see `MIN_LABEL_HORIZONTAL_CLEARANCE`/`MIN_LABEL_VERTICAL_CLEARANCE`
/// for why shrinking the canvas around a constant-size label (rather than
/// shrinking the label itself) is what raises small-cohort legibility.
const LABEL_FONT_SIZE: f64 = 12.0;
/// Horizontal canvas clearance beyond the ring at `MAX_VALIDATED_COHORT_SIZE`,
/// reserved for label text growing sideways from start/end-anchored nodes.
/// Sized for the longest expected single-line label (a ~15-char half of a
/// double-barrel name ≈ 112px at `LABEL_FONT_SIZE`, see the name-splitting
/// comment below) plus safety margin.
const MAX_LABEL_HORIZONTAL_CLEARANCE: f64 = 226.0;
/// Horizontal clearance at the smallest possible cohort. RV-06: the fixed
/// (cohort-size-independent) worst-case clearance above dominates a small
/// cohort's canvas — a small ring occupies only ~26% of it, reading as mostly
/// dead space. This floor still comfortably covers the same worst-case label
/// width (label length doesn't shrink with fewer participants) while letting
/// the canvas shrink close to proportionally with the ring, so small cohorts
/// fill their canvas about as fully as the validated 12-node cohort does.
const MIN_LABEL_HORIZONTAL_CLEARANCE: f64 = 140.0;
/// Vertical canvas clearance beyond the ring at `MAX_VALIDATED_COHORT_SIZE` —
/// smaller than the horizontal clearance because top/bottom (middle-anchored)
/// labels only need two-line-height room, not full text width.
const MAX_LABEL_VERTICAL_CLEARANCE: f64 = 151.0;
/// Vertical clearance at the smallest possible cohort (RV-06 rationale as
/// `MIN_LABEL_HORIZONTAL_CLEARANCE`). A two-line label needs roughly one line
/// height above and below its anchor plus font ascent/descent — comfortably
/// under this floor — so shrinking it here carries no clipping risk.
const MIN_LABEL_VERTICAL_CLEARANCE: f64 = 55.0;

/// Linearly interpolate a geometry value from `at_min_cohort` (at
/// `MIN_COHORT_SIZE`) to `at_max_cohort` (at `MAX_VALIDATED_COHORT_SIZE`),
/// clamping outside that band. Shared by every per-cohort geometry dimension
/// (ring radius, node radius, label clearances) so a cohort's whole canvas
/// scales together rather than just one dimension of it (RV-06).
#[allow(clippy::cast_precision_loss)]
fn scale_for_cohort_size(n: usize, at_min_cohort: f64, at_max_cohort: f64) -> f64 {
    if n >= MAX_VALIDATED_COHORT_SIZE {
        return at_max_cohort;
    }
    // n < MAX_VALIDATED_COHORT_SIZE here (early return above), so both operands
    // are single-digit — the usize→f64 casts are exact, no precision loss.
    let span = (MAX_VALIDATED_COHORT_SIZE - MIN_COHORT_SIZE) as f64;
    let steps_past_smallest = n.saturating_sub(MIN_COHORT_SIZE) as f64;
    let progress = steps_past_smallest / span;
    at_min_cohort + progress * (at_max_cohort - at_min_cohort)
}

/// Ring radius for a cohort of `n` participants: grows from `MIN_RING_RADIUS`
/// at the smallest possible cohort to `MAX_RING_RADIUS` at
/// `MAX_VALIDATED_COHORT_SIZE`, so a small cycle doesn't inherit a canvas sized
/// for the largest cohort.
fn ring_radius_for_cohort_size(n: usize) -> f64 {
    scale_for_cohort_size(n, MIN_RING_RADIUS, MAX_RING_RADIUS)
}

/// Node dot radius for a cohort of `n` participants — see `MIN_NODE_RADIUS`.
fn node_radius_for_cohort_size(n: usize) -> f64 {
    scale_for_cohort_size(n, MIN_NODE_RADIUS, MAX_NODE_RADIUS)
}

/// Horizontal label clearance for a cohort of `n` participants — see
/// `MIN_LABEL_HORIZONTAL_CLEARANCE`.
fn label_horizontal_clearance_for_cohort_size(n: usize) -> f64 {
    scale_for_cohort_size(
        n,
        MIN_LABEL_HORIZONTAL_CLEARANCE,
        MAX_LABEL_HORIZONTAL_CLEARANCE,
    )
}

/// Vertical label clearance for a cohort of `n` participants — see
/// `MIN_LABEL_VERTICAL_CLEARANCE`.
fn label_vertical_clearance_for_cohort_size(n: usize) -> f64 {
    scale_for_cohort_size(
        n,
        MIN_LABEL_VERTICAL_CLEARANCE,
        MAX_LABEL_VERTICAL_CLEARANCE,
    )
}

/// Every per-cohort SVG dimension for a cycle-ring, derived once so rendering
/// and tests share one authoritative geometry (no duplicated arithmetic).
#[derive(Clone, Copy)]
struct CycleGeometry {
    ring_radius: f64,
    node_radius: f64,
    half_width: f64,
    half_height: f64,
}

impl CycleGeometry {
    fn for_cohort_size(n: usize) -> Self {
        let ring_radius = ring_radius_for_cohort_size(n);
        let node_radius = node_radius_for_cohort_size(n);
        let label_horizontal_clearance = label_horizontal_clearance_for_cohort_size(n);
        let label_vertical_clearance = label_vertical_clearance_for_cohort_size(n);
        Self {
            ring_radius,
            node_radius,
            half_width: ring_radius + node_radius + LABEL_RADIAL_GAP + label_horizontal_clearance,
            half_height: ring_radius + node_radius + LABEL_RADIAL_GAP + label_vertical_clearance,
        }
    }

    fn viewbox_width(self) -> f64 {
        self.half_width * 2.0
    }

    fn viewbox_height(self) -> f64 {
        self.half_height * 2.0
    }
}

/// Render a single cohort cycle as an SVG ring.
///
/// Precision loss is acceptable for visual positioning of <20 participants.
#[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
fn render_cycle_ring(chain: &[AssignmentLink], cohort_num: usize, score: u32) -> impl IntoView {
    let n = chain.len();
    if n == 0 {
        return view! { <div></div> }.into_any();
    }

    // The whole canvas — ring radius, node size, and label clearances — scales
    // together with cohort size (CycleGeometry), not just the ring radius: a
    // small cohort gets a tight viewBox with a constant label font size (so
    // effective on-screen labels grow, not shrink) while cohorts at
    // MAX_VALIDATED_COHORT_SIZE+ reproduce the previously validated geometry.
    let geometry = CycleGeometry::for_cohort_size(n);
    let ring_radius = geometry.ring_radius;
    let node_radius = geometry.node_radius;
    let half_width = geometry.half_width;
    let half_height = geometry.half_height;
    let center_x = half_width;
    let center_y = half_height;
    let viewbox_width = geometry.viewbox_width();
    let viewbox_height = geometry.viewbox_height();

    let positions = compute_circle_positions(n, ring_radius, center_x, center_y);

    let arrows = (0..n)
        .map(|i| {
            let (x1, y1) = positions[i];
            let (x2, y2) = positions[(i + 1) % n];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dist = (dx * dx + dy * dy).sqrt();
            let ux = dx / dist;
            let uy = dy / dist;
            let margin = node_radius + ARROW_CLEARANCE;
            let start_x = x1 + ux * margin;
            let start_y = y1 + uy * margin;
            let end_x = x2 - ux * margin;
            let end_y = y2 - uy * margin;

            view! {
                <line
                    x1=start_x
                    y1=start_y
                    x2=end_x
                    y2=end_y
                    stroke="currentColor"
                    stroke-width="1.5"
                    marker-end="url(#arrowhead)"
                    opacity="0.5"
                />
            }
        })
        .collect_view();

    let nodes = chain
        .iter()
        .enumerate()
        .map(|(i, link)| {
            use std::f64::consts::{PI, TAU};
            let (cx, cy) = positions[i];
            let name = link.sender_name.clone();
            let sanitized = name.to_lowercase().replace(' ', "-");
            let testid = format!("node-{sanitized}");
            let user_id = link.sender_id.clone();

            // Radial angle of this node from center (0 = right, -PI/2 = top).
            let node_angle = (i as f64 / n as f64) * TAU - (PI / 2.0);

            // Place the label anchor point radially beyond the node edge.
            let label_radius = ring_radius + node_radius + LABEL_RADIAL_GAP;
            let label_x = center_x + label_radius * node_angle.cos();
            let label_y = center_y + label_radius * node_angle.sin();

            // Choose SVG text-anchor so the label grows away from the ring center,
            // preventing overlap with the adjacent node's label on the same side.
            let degrees_from_vertical = (node_angle.to_degrees().abs() % 180.0 - 90.0).abs();
            let anchor = if degrees_from_vertical < VERTICAL_AXIS_TOLERANCE_DEGREES {
                // Near the top or bottom: center-anchor avoids left/right drift.
                "middle"
            } else if node_angle.cos() > 0.0 {
                // Right half of ring: text grows rightward from anchor.
                "start"
            } else {
                // Left half of ring: text grows leftward from anchor.
                "end"
            };

            // Split "Given-Name Surname-Name" into two lines at the first space.
            // Each line of a 30-char double-barrel name becomes ~15 chars ≈ 112px
            // at LABEL_FONT_SIZE — fits within the per-node arc lane at n=15.
            let (line1, line2) = name
                .find(' ')
                .map_or((&name[..], ""), |pos| (&name[..pos], &name[pos + 1..]));

            let line_height = 15.0_f64;
            let line1_y = if line2.is_empty() {
                label_y
            } else {
                label_y - line_height / 2.0
            };
            let line2_y = line1_y + line_height;

            view! {
                <g>
                    <title>{name.clone()}</title>
                    <circle
                        cx=cx
                        cy=cy
                        r=node_radius
                        fill="var(--color-accent)"
                        stroke="var(--color-surface-raised)"
                        stroke-width="2"
                        data-testid=testid
                        data-user-id=user_id
                    />
                    <text
                        text-anchor=anchor
                        font-size=LABEL_FONT_SIZE
                        font-weight="600"
                        fill="var(--color-text)"
                    >
                        <tspan x=label_x y=line1_y>{line1.to_owned()}</tspan>
                        {(!line2.is_empty()).then(|| view! {
                            <tspan x=label_x y=line2_y>{line2.to_owned()}</tspan>
                        })}
                    </text>
                </g>
            }
        })
        .collect_view();

    view! {
        <figure class="cycle-viz-container" data-testid="cycle-visualization">
            <figcaption class="sr-only">
                "Assignment cycle " {cohort_num} ": " {n} " participants (score: " {score} ")"
            </figcaption>
            <svg
                viewBox=format!("0 0 {viewbox_width} {viewbox_height}")
                style=format!("aspect-ratio: {viewbox_width} / {viewbox_height}")
                class="cycle-viz"
                role="img"
                aria-label=format!("Assignment cycle {cohort_num}: {n} participants")
            >
                <defs>
                    <marker
                        id="arrowhead"
                        viewBox="0 0 10 7"
                        refX="10"
                        refY="3.5"
                        markerWidth="8"
                        markerHeight="6"
                        orient="auto-start-reverse"
                    >
                        <polygon points="0 0, 10 3.5, 0 7" fill="currentColor" />
                    </marker>
                </defs>
                {arrows}
                {nodes}
            </svg>
        </figure>
    }
    .into_any()
}

// ── Swap form sub-component ────────────────────────────────────────────────────

#[component]
fn SwapFormSection(
    swap_action: ServerAction<SwapAssignment>,
    season_id: String,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
    links: Vec<AssignmentLink>,
) -> impl IntoView {
    let swap_pending = swap_action.pending();
    let options_a = links.clone();
    let options_b = links;
    view! {
        <section data-testid="override-available">
            <h3>{t!(i18n, assignments_swap_title)}</h3>
            <p>{t!(i18n, assignments_swap_description)}</p>
            <leptos::form::ActionForm action=swap_action>
                <input type="hidden" name="season_id" value=season_id />
                <div class="field">
                    <label class="field-label" for="sender-a">
                        {t!(i18n, assignments_sender_a_label)}
                    </label>
                    <select
                        class="field-input"
                        id="sender-a"
                        name="sender_a"
                        data-testid="sender-a-input"
                        required=true
                        aria-invalid=move || swap_action.value().get().and_then(Result::err).map(|_| "true")
                        aria-describedby="action-error"
                    >
                        <option value="">{t!(i18n, assignments_select_sender)}</option>
                        {options_a
                            .iter()
                            .map(|link| {
                                let id = link.sender_id.clone();
                                let name = link.sender_name.clone();
                                view! { <option value=id>{name}</option> }
                            })
                            .collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label class="field-label" for="sender-b">
                        {t!(i18n, assignments_sender_b_label)}
                    </label>
                    <select
                        class="field-input"
                        id="sender-b"
                        name="sender_b"
                        data-testid="sender-b-input"
                        required=true
                        aria-invalid=move || swap_action.value().get().and_then(Result::err).map(|_| "true")
                        aria-describedby="action-error"
                    >
                        <option value="">{t!(i18n, assignments_select_sender)}</option>
                        {options_b
                            .iter()
                            .map(|link| {
                                let id = link.sender_id.clone();
                                let name = link.sender_name.clone();
                                view! { <option value=id>{name}</option> }
                            })
                            .collect_view()}
                    </select>
                </div>
                // Secondary variant (FU-20): reverses S3's per-form primary call.
                // FU-20 grades emphasis per PAGE, not per form — the season's
                // phase-advance control is the sole primary; this form's own sole
                // CTA still reads clearly as secondary, without competing for the
                // page's one orange action.
                <button
                    class="btn"
                    data-variant="secondary"
                    type="submit"
                    data-testid="swap-button"
                    disabled=move || swap_pending.get() || !hydrated.get()
                    attr:aria-busy=move || swap_pending.get().then_some("true")
                >
                    {t!(i18n, assignments_apply_button)}
                </button>
            </leptos::form::ActionForm>
        </section>
    }
}

// ── Participant section sub-components ────────────────────────────────────────

#[component]
fn InviteCodesSection(
    generate_invite_action: ServerAction<GenerateInviteCode>,
    revoke_invite_action: ServerAction<RevokeInviteCode>,
    invite_codes: Resource<Result<Vec<InviteCodeRow>, ServerFnError>>,
    distributor_options: Resource<Result<Vec<DistributorOption>, ServerFnError>>,
    hydrated: ReadSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let generate_pending = generate_invite_action.pending();
    let revoke_pending = revoke_invite_action.pending();
    let (filter_query, set_filter_query) = signal(String::new());
    // generate_invite_code has exactly one input (distributor), so any error it
    // returns belongs to this field's own local `.field-error` — no discriminant
    // wire format needed, unlike the multi-field create-season/onboarding forms.
    let distributor_error = move || {
        generate_invite_action
            .value()
            .get()
            .and_then(Result::err)
            .map(|e| strip_server_error_prefix(&e))
    };

    view! {
        <section class="admin-section" data-testid="invite-codes-section">
            // Section heading, peer to "Список учасників" (both under "Учасники").
            // Was `.overline-label` — an overline reads as a label ABOVE a heading,
            // inverting hierarchy against its plain-h2 sibling (S5). Aligned to a
            // plain section h2 so the two sub-sections share one heading level.
            <h2>{t!(i18n, admin_invite_codes_section_title)}</h2>

            // ── Generate subsection ───────────────────────────────────────────
            <h3 class="overline-label">{t!(i18n, admin_invite_codes_generate_subsection_title)}</h3>

            <Suspense fallback=|| ()>
                {move || {
                    distributor_options
                        .get()
                        .map(|result| match result {
                            Err(e) => {
                                view! { <p class="alert">{strip_server_error_prefix(&e)}</p> }
                                    .into_any()
                            }
                            Ok(options) => {
                                let options_for_view = options.clone();
                                view! {
                                    <leptos::form::ActionForm action=generate_invite_action>
                                        <div class="field">
                                            <label class="field-label" for="distributor-id">
                                                {t!(i18n, admin_invite_codes_distributor_label)}
                                            </label>
                                            <select
                                                class="field-input"
                                                id="distributor-id"
                                                name="distributor_id"
                                                data-testid="distributor-select"
                                                required=true
                                                aria-describedby="distributor-id-error"
                                                aria-invalid=move || distributor_error().is_some().then_some("true")
                                            >
                                                <option value="">
                                                    {t!(i18n, assignments_select_sender)}
                                                </option>
                                                {options_for_view
                                                    .iter()
                                                    .map(|opt| {
                                                        let id = opt.id.to_string();
                                                        let name = opt.name.clone();
                                                        view! { <option value=id>{name}</option> }
                                                    })
                                                    .collect_view()}
                                            </select>
                                            <p
                                                id="distributor-id-error"
                                                class="field-error"
                                                aria-live="assertive"
                                                data-testid="distributor-id-error"
                                            >
                                                {move || distributor_error()}
                                            </p>
                                        </div>
                                        // Secondary variant (FU-20): card-local
                                        // routine create action, calm relative to
                                        // the page's one phase-transition primary.
                                        <button
                                            class="btn"
                                            data-variant="secondary"
                                            type="submit"
                                            data-testid="generate-code-button"
                                            disabled=move || generate_pending.get() || !hydrated.get()
                                            attr:aria-busy=move || generate_pending.get().then_some("true")
                                        >
                                            {t!(i18n, admin_invite_codes_generate_button)}
                                        </button>
                                    </leptos::form::ActionForm>
                                }
                                .into_any()
                            }
                        })
                }}
            </Suspense>

            // ── Generated code display ────────────────────────────────────────
            <Show when=move || {
                generate_invite_action.value().get().is_some_and(|r| r.is_ok())
            }>
                <div
                    class="flex items-center gap-(--density-space-sm) mt-(--density-space-sm)"
                    data-testid="generated-code-display"
                    role="status"
                    aria-live="polite"
                >
                    <span class="text-sm font-semibold">
                        {t!(i18n, admin_invite_codes_generated_label)}
                    </span>
                    <span
                        class="invite-code-card-code"
                        data-testid="generated-code-value"
                    >
                        {move || {
                            generate_invite_action
                                .value()
                                .get()
                                .and_then(Result::ok)
                                .unwrap_or_default()
                        }}
                    </span>
                </div>
            </Show>

            // ── Invite code list subsection ───────────────────────────────────
            <h3 class="overline-label mt-(--density-space-lg)">{t!(i18n, admin_invite_codes_list_title)}</h3>

            <div class="field">
                <label class="field-label" for="invite-code-filter">
                    {t!(i18n, admin_invite_codes_filter_label)}
                </label>
                <input
                    class="field-input"
                    id="invite-code-filter"
                    type="text"
                    data-testid="invite-code-filter-input"
                    placeholder=move || t_string!(i18n, admin_invite_codes_filter_placeholder)
                    prop:value=move || filter_query.get()
                    on:input=move |ev| {
                        set_filter_query.set(event_target_value(&ev));
                    }
                />
            </div>

            <div data-testid="invite-code-list">
                <Suspense fallback=|| ()>
                    {move || {
                        invite_codes
                            .get()
                            .map(|result| match result {
                                Err(e) => {
                                    view! {
                                        <p class="alert">{strip_server_error_prefix(&e)}</p>
                                    }
                                    .into_any()
                                }
                                Ok(codes) if codes.is_empty() => {
                                    view! {
                                        <div
                                            class="empty-state"
                                            data-testid="invite-code-empty-state"
                                        >
                                            <p class="empty-state-headline">
                                                {t!(i18n, admin_invite_codes_empty_state)}
                                            </p>
                                            <p class="empty-state-body">
                                                {t!(i18n, admin_invite_codes_empty_state_body)}
                                            </p>
                                        </div>
                                    }
                                    .into_any()
                                }
                                Ok(codes) => {
                                    view! {
                                        <ul class="invite-code-list">
                                            <For
                                                each=move || {
                                                    let query = filter_query.get().to_lowercase();
                                                    if query.is_empty() {
                                                        return codes.clone();
                                                    }
                                                    codes
                                                        .iter()
                                                        .filter(|c| {
                                                            c.code.to_lowercase().contains(&query)
                                                                || c.distributor_name.to_lowercase().contains(&query)
                                                                || matches_invite_status(c.status, &query)
                                                                || c.redeemer_name
                                                                    .as_deref()
                                                                    .is_some_and(|n| n.to_lowercase().contains(&query))
                                                        })
                                                        .cloned()
                                                        .collect::<Vec<_>>()
                                                }
                                                key=|c| c.id
                                                let:code
                                            >
                                                <li class="invite-code-card" data-testid="invite-code-row">
                                                    // Code — primary identifier
                                                    <span
                                                        class="invite-code-card-code"
                                                        data-testid="invite-code-cell"
                                                    >
                                                        {code.code.clone()}
                                                    </span>
                                                    // Distributor — canonical label/value pair (RV-05):
                                                    // routed through .info-list so this meta block
                                                    // renders with the same overline-label + value
                                                    // treatment as every other label/value pair in the
                                                    // app, instead of a bare, unlabeled line.
                                                    <dl
                                                        class="info-list invite-code-card-meta"
                                                        data-testid="invite-code-distributor-cell"
                                                    >
                                                        <div class="info-item">
                                                            <dt class="info-label">
                                                                {t!(
                                                                    i18n,
                                                                    admin_invite_codes_distributor_label
                                                                )}
                                                            </dt>
                                                            <dd class="info-value">
                                                                {code.distributor_name.clone()}
                                                            </dd>
                                                        </div>
                                                    </dl>
                                                    // Status badge
                                                    <span data-testid="invite-code-status-cell">
                                                        {match code.status {
                                                            InviteCodeStatus::Unused => {
                                                                view! {
                                                                    <span
                                                                        class="badge"
                                                                        data-testid="invite-code-status-badge"
                                                                        data-status="unused"
                                                                    >
                                                                        {t!(
                                                                            i18n,
                                                                            admin_invite_codes_status_unused
                                                                        )}
                                                                    </span>
                                                                }
                                                                .into_any()
                                                            }
                                                            InviteCodeStatus::Used => {
                                                                view! {
                                                                    <span
                                                                        class="badge"
                                                                        data-testid="invite-code-status-badge"
                                                                        data-status="used"
                                                                    >
                                                                        {t!(
                                                                            i18n,
                                                                            admin_invite_codes_status_used
                                                                        )}
                                                                    </span>
                                                                }
                                                                .into_any()
                                                            }
                                                            InviteCodeStatus::Revoked => {
                                                                view! {
                                                                    <span
                                                                        class="badge"
                                                                        data-testid="invite-code-status-badge"
                                                                        data-status="revoked"
                                                                    >
                                                                        {t!(
                                                                            i18n,
                                                                            admin_invite_codes_status_revoked
                                                                        )}
                                                                    </span>
                                                                }
                                                                .into_any()
                                                            }
                                                        }}
                                                    </span>
                                                    // Redeemer + timestamp (only when used).
                                                    // Name and date on separate lines so a long
                                                    // double-barrel name wraps within its own row
                                                    // instead of pushing the date to a second line
                                                    // and jaggedly varying card height (OV L11).
                                                    <span
                                                        class="invite-code-card-redeemer"
                                                        data-testid="invite-code-redeemer-cell"
                                                    >
                                                        {match (code.redeemer_name.clone(), code.redeemed_at.clone()) {
                                                            (Some(name), Some(date_str)) => {
                                                                view! {
                                                                    <span class="invite-code-card-redeemer-name">
                                                                        {name}
                                                                    </span>
                                                                    <small class="invite-code-card-redeemer-date">
                                                                        {date_str}
                                                                    </small>
                                                                }
                                                                    .into_any()
                                                            }
                                                            (Some(name), None) => {
                                                                view! {
                                                                    <span class="invite-code-card-redeemer-name">
                                                                        {name}
                                                                    </span>
                                                                }
                                                                    .into_any()
                                                            }
                                                            _ => ().into_any(),
                                                        }}
                                                    </span>
                                                    // Revoke action (only for unused codes)
                                                    <span class="invite-code-card-action">
                                                        {if code.status == InviteCodeStatus::Unused {
                                                            let code_id = code.id.to_string();
                                                            view! {
                                                                <leptos::form::ActionForm action=revoke_invite_action>
                                                                    <input
                                                                        type="hidden"
                                                                        name="id"
                                                                        value=code_id
                                                                    />
                                                                    // Secondary variant (FU-20): revoke is frequent
                                                                    // and low-consequence — demoted off
                                                                    // destructive-red so red stays a real danger
                                                                    // signal reserved for cancel-confirm. Intent
                                                                    // is carried by the label + status badge.
                                                                    <button
                                                                        class="btn"
                                                                        data-variant="secondary"
                                                                        data-size="sm"
                                                                        type="submit"
                                                                        data-testid="invite-code-revoke-button"
                                                                        disabled=move || revoke_pending.get() || !hydrated.get()
                                                                        attr:aria-busy=move || revoke_pending.get().then_some("true")
                                                                    >
                                                                        {t!(
                                                                            i18n,
                                                                            admin_invite_codes_revoke_button
                                                                        )}
                                                                    </button>
                                                                </leptos::form::ActionForm>
                                                            }
                                                            .into_any()
                                                        } else {
                                                            ().into_any()
                                                        }}
                                                    </span>
                                                </li>
                                            </For>
                                        </ul>
                                    }
                                    .into_any()
                                }
                            })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

fn matches_invite_status(status: InviteCodeStatus, query: &str) -> bool {
    let status_text = match status {
        InviteCodeStatus::Unused => "unused",
        InviteCodeStatus::Used => "used",
        InviteCodeStatus::Revoked => "revoked",
    };
    status_text.contains(query)
}

#[component]
fn ParticipantListSection(
    participants: Resource<Result<Vec<ParticipantSummary>, ServerFnError>>,
    deactivate_action: ServerAction<DeactivateParticipant>,
) -> impl IntoView {
    let i18n = use_i18n();
    let hydrated = use_hydrated();

    view! {
        <div data-testid="participant-list">
            <Suspense fallback=move || view! { <SkeletonFallback /> }>
                {move || {
                    participants
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="alert">{strip_server_error_prefix(&e)}</p> }.into_any(),
                            Ok(list) => {
                                if list.is_empty() {
                                    view! {
                                        <div class="empty-state" data-testid="empty-state">
                                            <p class="empty-state-headline">{t!(i18n, admin_participants_empty_state)}</p>
                                            <p class="empty-state-body">
                                                {t!(i18n, admin_participants_empty_state_body)}
                                            </p>
                                        </div>
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <div class="data-table-wrapper">
                                            <table class="data-table">
                                                <thead>
                                                    <tr>
                                                        <th>{t!(i18n, participants_table_name)}</th>
                                                        <th>{t!(i18n, participants_table_phone)}</th>
                                                        <th>{t!(i18n, participants_table_status)}</th>
                                                        <th>{t!(i18n, participants_table_actions)}</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || list.clone()
                                                        key=|p| p.id
                                                        children=move |p| {
                                                            let uid_str = p.id.to_string();
                                                            let active = matches!(
                                                                p.status,
                                                                crate::types::UserStatus::Active
                                                            );
                                                            view! {
                                                                <tr data-testid="participant-row">
                                                                    <td data-testid="participant-name-cell">
                                                                        {p.name.clone()}
                                                                    </td>
                                                                    <td>{p.phone.clone()}</td>
                                                                    <td>
                                                                        {move || {
                                                                            if active {
                                                                                view! {
                                                                                    <span
                                                                                        class="badge"
                                                                                        data-status="active"
                                                                                    >
                                                                                        {t!(i18n, participants_status_active)}
                                                                                    </span>
                                                                                }
                                                                                .into_any()
                                                                            } else {
                                                                                view! {
                                                                                    <span
                                                                                        class="badge"
                                                                                        data-status="inactive"
                                                                                    >
                                                                                        {t!(i18n, participants_status_deactivated)}
                                                                                    </span>
                                                                                }
                                                                                .into_any()
                                                                            }
                                                                        }}
                                                                    </td>
                                                                    <td>
                                                                        {
                                                                            let deactivate_pending = deactivate_action.pending();
                                                                            if active {
                                                                                view! {
                                                                                    <leptos::form::ActionForm action=deactivate_action>
                                                                                        <input
                                                                                            type="hidden"
                                                                                            name="user_id"
                                                                                            value=uid_str
                                                                                        />
                                                                                        // Secondary variant (FU-20): same
                                                                                        // rationale as invite-code revoke —
                                                                                        // frequent, low-consequence, kept
                                                                                        // off destructive-red.
                                                                                        <button
                                                                                            class="btn"
                                                                                            data-variant="secondary"
                                                                                            data-size="sm"
                                                                                            type="submit"
                                                                                            data-testid="deactivate-button"
                                                                                            disabled=move || {
                                                                                                deactivate_pending.get() || !hydrated.get()
                                                                                            }
                                                                                            attr:aria-busy=move || deactivate_pending.get().then_some("true")
                                                                                        >
                                                                                            {move || if deactivate_pending.get() {
                                                                                                t!(i18n, admin_participant_deactivating_loading).into_any()
                                                                                            } else {
                                                                                                t!(
                                                                                                    i18n,
                                                                                                    participants_deactivate_button
                                                                                                )
                                                                                                .into_any()
                                                                                            }}
                                                                                        </button>
                                                                                    </leptos::form::ActionForm>
                                                                                }
                                                                                .into_any()
                                                                            } else {
                                                                                // Deactivated rows carry no action. The STATUS
                                                                                // column already shows the inactive badge, so a
                                                                                // second gray pill here would double-convey and
                                                                                // read as a disabled-button false affordance
                                                                                // (S2). A muted em-dash marks the cell as
                                                                                // intentionally empty. testid preserved for E2E.
                                                                                view! {
                                                                                    <span
                                                                                        class="inactive-marker"
                                                                                        aria-hidden="true"
                                                                                        data-testid="inactive-status"
                                                                                    >
                                                                                        "—"
                                                                                    </span>
                                                                                }
                                                                                .into_any()
                                                                            }
                                                                        }
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CycleGeometry, MAX_LABEL_HORIZONTAL_CLEARANCE, MAX_LABEL_VERTICAL_CLEARANCE,
        MAX_NODE_RADIUS, MAX_RING_RADIUS, MAX_VALIDATED_COHORT_SIZE, MIN_COHORT_SIZE,
        MIN_LABEL_HORIZONTAL_CLEARANCE, MIN_LABEL_VERTICAL_CLEARANCE, MIN_NODE_RADIUS,
        MIN_RING_RADIUS, label_horizontal_clearance_for_cohort_size,
        label_vertical_clearance_for_cohort_size, node_radius_for_cohort_size,
        ring_radius_for_cohort_size,
    };

    /// Two values are equal within sub-pixel tolerance (float positioning math).
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn radius_at_min_cohort_is_min_radius() {
        assert!(approx_eq(
            ring_radius_for_cohort_size(MIN_COHORT_SIZE),
            MIN_RING_RADIUS
        ));
    }

    #[test]
    fn radius_below_min_cohort_clamps_to_min_radius() {
        // Cohorts never fall below MIN_COHORT_SIZE, but the fn must not underflow.
        for n in 0..MIN_COHORT_SIZE {
            assert!(approx_eq(ring_radius_for_cohort_size(n), MIN_RING_RADIUS));
        }
    }

    #[test]
    fn radius_at_validated_cohort_matches_prior_fixed_geometry() {
        // Load-bearing invariant: at MAX_VALIDATED_COHORT_SIZE the radius is
        // exactly MAX_RING_RADIUS (190), reproducing the previously-validated
        // 900x750 canvas so the one collision-free layout is preserved.
        assert!(approx_eq(
            ring_radius_for_cohort_size(MAX_VALIDATED_COHORT_SIZE),
            MAX_RING_RADIUS
        ));
    }

    #[test]
    fn radius_above_validated_cohort_saturates_at_max_radius() {
        for n in [MAX_VALIDATED_COHORT_SIZE + 1, 15, 100] {
            assert!(approx_eq(ring_radius_for_cohort_size(n), MAX_RING_RADIUS));
        }
    }

    #[test]
    fn radius_grows_strictly_and_stays_in_range_across_the_band() {
        let mut previous = ring_radius_for_cohort_size(MIN_COHORT_SIZE);
        for n in (MIN_COHORT_SIZE + 1)..=MAX_VALIDATED_COHORT_SIZE {
            let current = ring_radius_for_cohort_size(n);
            assert!(current > previous, "radius must increase from n-1 to n");
            assert!((MIN_RING_RADIUS..=MAX_RING_RADIUS).contains(&current));
            previous = current;
        }
    }

    #[test]
    fn interpolation_is_linear_equal_steps_give_equal_deltas() {
        // A one-participant increase changes the radius by a constant amount
        // anywhere inside the band — the defining property of linear interpolation.
        let delta_low = ring_radius_for_cohort_size(5) - ring_radius_for_cohort_size(4);
        let delta_high = ring_radius_for_cohort_size(9) - ring_radius_for_cohort_size(8);
        assert!(approx_eq(delta_low, delta_high));
    }

    #[test]
    fn node_radius_and_clearances_are_wired_to_their_own_bounds() {
        // These wrap the same generic `scale_for_cohort_size` already exercised
        // above via ring radius — this only confirms each wrapper reaches its
        // own min/max constants, not a re-test of the interpolation itself.
        assert!(approx_eq(
            node_radius_for_cohort_size(MIN_COHORT_SIZE),
            MIN_NODE_RADIUS
        ));
        assert!(approx_eq(
            node_radius_for_cohort_size(MAX_VALIDATED_COHORT_SIZE),
            MAX_NODE_RADIUS
        ));
        assert!(approx_eq(
            label_horizontal_clearance_for_cohort_size(MIN_COHORT_SIZE),
            MIN_LABEL_HORIZONTAL_CLEARANCE
        ));
        assert!(approx_eq(
            label_horizontal_clearance_for_cohort_size(MAX_VALIDATED_COHORT_SIZE),
            MAX_LABEL_HORIZONTAL_CLEARANCE
        ));
        assert!(approx_eq(
            label_vertical_clearance_for_cohort_size(MIN_COHORT_SIZE),
            MIN_LABEL_VERTICAL_CLEARANCE
        ));
        assert!(approx_eq(
            label_vertical_clearance_for_cohort_size(MAX_VALIDATED_COHORT_SIZE),
            MAX_LABEL_VERTICAL_CLEARANCE
        ));
    }

    #[test]
    fn validated_cohort_geometry_is_unregressed() {
        // RV-06 acceptance: the 12-node cohort must reproduce the exact
        // previously-validated 900x750 canvas — this fix must change nothing
        // at or above MAX_VALIDATED_COHORT_SIZE.
        let geometry = CycleGeometry::for_cohort_size(MAX_VALIDATED_COHORT_SIZE);
        assert!(approx_eq(geometry.viewbox_width(), 900.0));
        assert!(approx_eq(geometry.viewbox_height(), 750.0));
    }

    #[test]
    fn smallest_cohort_ring_fills_a_comparable_share_of_its_canvas() {
        // RV-06: before this fix, a 3-node ring occupied only ~33% of its
        // canvas height (top+bottom dead space) because the fixed label
        // clearances dominated a shrunk ring. Scaling node radius and label
        // clearances down alongside the ring closes that gap to within a
        // modest band of the validated cohort's own fill ratio, without
        // shrinking clearances below what the same worst-case label text
        // (see MIN_LABEL_HORIZONTAL_CLEARANCE) needs regardless of cohort size.
        let small = CycleGeometry::for_cohort_size(MIN_COHORT_SIZE);
        let validated = CycleGeometry::for_cohort_size(MAX_VALIDATED_COHORT_SIZE);

        let vertical_fill_ratio = |g: CycleGeometry| (g.ring_radius * 2.0) / g.viewbox_height();
        let small_fill = vertical_fill_ratio(small);
        let validated_fill = vertical_fill_ratio(validated);

        assert!(
            small_fill > 0.45,
            "smallest cohort should fill well over the pre-fix ~33%, got {small_fill}"
        );
        assert!(
            (small_fill - validated_fill).abs() < 0.10,
            "small cohort ({small_fill}) and validated cohort ({validated_fill}) fill ratios \
             should stay within a modest band of each other"
        );
    }
}

use crate::i18n::i18n::{t, use_i18n};
use crate::types::Phase;
use leptos::prelude::*;

/// Phase label for the given phase, resolved from i18n.
fn phase_label(i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>, phase: Phase) -> AnyView {
    match phase {
        Phase::Enrollment => t!(i18n, season_phase_enrollment).into_any(),
        Phase::Preparation => t!(i18n, season_phase_preparation).into_any(),
        Phase::Assignment => t!(i18n, season_phase_assignment).into_any(),
        Phase::Delivery => t!(i18n, season_phase_delivery).into_any(),
        Phase::Complete => t!(i18n, season_phase_complete).into_any(),
        Phase::Cancelled => t!(i18n, season_phase_cancelled).into_any(),
    }
}

/// Stepper showing season progress through phases.
///
/// Two responsive forms share one `phase-stepper` testid:
/// - Desktop (≥640px): a horizontal 5-step strip with per-step status
///   (completed green check / current accent / locked gray / abandoned muted).
/// - Mobile (<640px): a compact single line "Крок N з 5: <phase>" — the
///   5-label Ukrainian strip does not fit 375px, so it is replaced rather
///   than shrunk. Cancelled shows a muted "Сезон скасовано" line.
///
/// Step number and labels derive from the phase enum — single source of truth.
///
/// Used on both admin dashboard and participant home.
#[component]
pub fn PhaseStepper(
    /// The current active phase
    current_phase: Phase,
) -> impl IntoView {
    let i18n = use_i18n();

    // Phase sequence (excludes Cancelled — it's not in the forward progression)
    let phases = [
        Phase::Enrollment,
        Phase::Preparation,
        Phase::Assignment,
        Phase::Delivery,
        Phase::Complete,
    ];

    let is_cancelled = current_phase == Phase::Cancelled;
    // Complete is a TERMINAL state, not an in-progress step. When the season is
    // Complete the final step must read as a done green check, not an orange
    // "current" step-5 ("still on step 5") — S14. is_complete promotes the
    // Complete step (and all before it) to "completed".
    let is_complete = current_phase == Phase::Complete;

    // Current step number (1-based) within the forward progression, for the compact form.
    // Position is 0..5, so +1 is 1..=5 — always fits i64 without truncation.
    let current_step: i64 = phases
        .iter()
        .position(|p| *p == current_phase)
        .map_or(1, |i| i64::try_from(i + 1).unwrap_or(1));

    // Compact (mobile) indicator: "Крок N з 5: <phase>" for active phases,
    // muted "Сезон скасовано" for cancelled.
    let compact = if is_cancelled {
        view! {
            <p
                class="stepper-compact"
                data-status="abandoned"
                aria-hidden="true"
            >
                {t!(i18n, season_stepper_cancelled)}
            </p>
        }
        .into_any()
    } else {
        let step = current_step;
        view! {
            <p class="stepper-compact" aria-hidden="true">
                {t!(i18n, season_stepper_compact_prefix, step = step)}
                " "
                {phase_label(i18n, current_phase)}
            </p>
        }
        .into_any()
    };

    view! {
        <nav aria-label="Season progress" data-testid="phase-stepper">
            {compact}
            <ol class="stepper stepper-strip">
                {phases
                    .into_iter()
                    .enumerate()
                    .flat_map(|(idx, phase)| {
                        // Cancelled: all phases show as "abandoned" — muted, not success-green.
                        // Complete (terminal): all phases (incl. Complete itself) show as
                        // "completed" green checks — the season is done, not "on step 5".
                        let status = if is_cancelled {
                            "abandoned"
                        } else if is_complete {
                            "completed"
                        } else {
                            match phase.cmp(&current_phase) {
                                std::cmp::Ordering::Less => "completed",
                                std::cmp::Ordering::Equal => "current",
                                std::cmp::Ordering::Greater => "locked",
                            }
                        };

                        // Get phase label from i18n
                        let label = phase_label(i18n, phase);

                        // Connector before step (except for first step).
                        // Green when the preceding step is completed; abandoned when cancelled.
                        // Uses <li> (not <div>) — <div> inside <ol> is invalid HTML
                        // and causes browser re-parenting that breaks sibling selectors.
                        let connector = if idx > 0 {
                            let connector_status = if is_cancelled {
                                "abandoned"
                            } else if is_complete || phases[idx - 1] < current_phase {
                                "completed"
                            } else {
                                "pending"
                            };
                            Some(view! {
                                <li
                                    class="step-connector"
                                    aria-hidden="true"
                                    data-status=connector_status
                                ></li>
                            })
                        } else {
                            None
                        };

                        // Marker content: checkmark for completed, dash for abandoned, number otherwise
                        let marker_content = if status == "completed" {
                            "\u{2713}".to_string()
                        } else if status == "abandoned" {
                            "\u{2013}".to_string()
                        } else {
                            (idx + 1).to_string()
                        };

                        // Step element
                        let step = view! {
                            <li
                                class="step"
                                data-status=status
                                attr:aria-current=if status == "current" { Some("step") } else { None }
                            >
                                <div class="step-marker" aria-hidden="true">
                                    {marker_content}
                                </div>
                                <div class="step-label">{label}</div>
                            </li>
                        };

                        // Return connector (if any) + step
                        [connector.map(IntoAny::into_any), Some(step.into_any())]
                    })
                    .flatten()
                    .collect_view()}
            </ol>
        </nav>
    }
}

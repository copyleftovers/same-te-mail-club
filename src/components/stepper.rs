use crate::i18n::i18n::{t, use_i18n};
use crate::types::Phase;
use leptos::prelude::*;

/// Horizontal stepper showing season progress through phases.
///
/// Renders all phases with visual status:
/// - Completed: green checkmark
/// - Current: orange/accent highlight
/// - Locked: dimmed/gray
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

    view! {
        <nav aria-label="Season progress" data-testid="phase-stepper">
            <ol class="stepper">
                {phases
                    .into_iter()
                    .enumerate()
                    .flat_map(|(idx, phase)| {
                        // Determine status
                        let status = match phase.cmp(&current_phase) {
                            std::cmp::Ordering::Less => "completed",
                            std::cmp::Ordering::Equal => "current",
                            std::cmp::Ordering::Greater => "locked",
                        };

                        // Get phase label from i18n
                        let label = match phase {
                            Phase::Enrollment => t!(i18n, season_phase_enrollment).into_any(),
                            Phase::Preparation => t!(i18n, season_phase_preparation).into_any(),
                            Phase::Assignment => t!(i18n, season_phase_assignment).into_any(),
                            Phase::Delivery => t!(i18n, season_phase_delivery).into_any(),
                            Phase::Complete => t!(i18n, season_phase_complete).into_any(),
                            Phase::Cancelled => t!(i18n, season_phase_cancelled).into_any(),
                        };

                        // Connector before step (except for first step).
                        // Green when the preceding step is completed.
                        // Uses <li> (not <div>) — <div> inside <ol> is invalid HTML
                        // and causes browser re-parenting that breaks sibling selectors.
                        let connector = if idx > 0 {
                            let prev_completed = phases[idx - 1] < current_phase;
                            let connector_status = if prev_completed { "completed" } else { "pending" };
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

                        // Marker content: checkmark for completed, number for others
                        let marker_content = if status == "completed" {
                            "\u{2713}".to_string()
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

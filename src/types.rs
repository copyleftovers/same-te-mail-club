use serde::{Deserialize, Serialize};

/// Season phase — mirrors `season_phase` Postgres enum.
/// Six phases: Enrollment → Preparation → Assignment → Delivery → Complete.
/// Cancelled reachable from any non-terminal.
/// Transition rules gathered here, not scattered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "season_phase", rename_all = "lowercase")
)]
pub enum Phase {
    Enrollment,
    Preparation,
    Assignment,
    Delivery,
    Complete,
    Cancelled,
}

impl Phase {
    /// Next phase in sequence.
    ///
    /// # Errors
    ///
    /// Returns `Err` if called on a terminal phase (`Complete` or `Cancelled`).
    pub fn try_advance(self) -> Result<Self, InvalidTransition> {
        match self {
            Self::Enrollment => Ok(Self::Preparation),
            Self::Preparation => Ok(Self::Assignment),
            Self::Assignment => Ok(Self::Delivery),
            Self::Delivery => Ok(Self::Complete),
            Self::Complete | Self::Cancelled => Err(InvalidTransition { from: self }),
        }
    }

    /// Returns true if this phase can be advanced.
    pub fn can_advance(self) -> bool {
        self.try_advance().is_ok()
    }

    /// Cancel from any non-terminal phase.
    ///
    /// # Errors
    ///
    /// Returns `Err` if called on a terminal phase (`Complete` or `Cancelled`).
    pub fn cancel(self) -> Result<Self, InvalidTransition> {
        if self.is_terminal() {
            Err(InvalidTransition { from: self })
        } else {
            Ok(Self::Cancelled)
        }
    }

    /// Returns true if this is a terminal phase (Complete or Cancelled).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Complete | Self::Cancelled)
    }
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Enrollment => "enrollment",
            Self::Preparation => "preparation",
            Self::Assignment => "assignment",
            Self::Delivery => "delivery",
            Self::Complete => "complete",
            Self::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

/// User role — mirrors `user_role` Postgres enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "user_role", rename_all = "lowercase")
)]
pub enum UserRole {
    Participant,
    Admin,
}

/// User account status — mirrors `user_status` Postgres enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "user_status", rename_all = "lowercase")
)]
pub enum UserStatus {
    Active,
    Deactivated,
}

/// Receipt status — mirrors `receipt_status` Postgres enum.
/// `NoResponse` = hasn't responded yet (default).
/// `Received` = actively confirmed receipt.
/// `NotReceived` = actively reported non-receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "receipt_status", rename_all = "snake_case")
)]
pub enum ReceiptStatus {
    NoResponse,
    Received,
    NotReceived,
}

/// Authenticated user data returned by `get_current_user` server function.
/// Lives in types.rs so it is available in both SSR and WASM builds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: uuid::Uuid,
    pub name: String,
    pub role: UserRole,
    pub onboarded: bool,
}

/// Error for invalid phase transitions. Kept in types.rs (shared)
/// so Phase methods can use it without SSR dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: Phase,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no valid transition from {}", self.from)
    }
}

impl std::error::Error for InvalidTransition {}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Phase transition chain (valid path)
    #[test]
    fn test_phase_advance_chain() {
        assert_eq!(Phase::Enrollment.try_advance(), Ok(Phase::Preparation));
        assert_eq!(Phase::Preparation.try_advance(), Ok(Phase::Assignment));
        assert_eq!(Phase::Assignment.try_advance(), Ok(Phase::Delivery));
        assert_eq!(Phase::Delivery.try_advance(), Ok(Phase::Complete));
    }

    // Story: Terminal phases cannot advance
    #[test]
    fn test_advance_from_complete_is_err() {
        assert!(Phase::Complete.try_advance().is_err());
    }

    #[test]
    fn test_advance_from_cancelled_is_err() {
        assert!(Phase::Cancelled.try_advance().is_err());
    }

    // Story: Cancel from any non-terminal returns Cancelled
    #[test]
    fn test_cancel_from_non_terminal() {
        let non_terminals = [
            Phase::Enrollment,
            Phase::Preparation,
            Phase::Assignment,
            Phase::Delivery,
        ];
        for phase in non_terminals {
            assert_eq!(
                phase.cancel(),
                Ok(Phase::Cancelled),
                "cancel() from {phase} should return Cancelled"
            );
        }
    }

    // Story: Cancel from terminal phases returns Err
    #[test]
    fn test_cancel_from_complete_is_err() {
        assert!(Phase::Complete.cancel().is_err());
    }

    #[test]
    fn test_cancel_from_cancelled_is_err() {
        assert!(Phase::Cancelled.cancel().is_err());
    }

    // Story: is_terminal() correctness
    #[test]
    fn test_is_terminal() {
        assert!(Phase::Complete.is_terminal());
        assert!(Phase::Cancelled.is_terminal());
        assert!(!Phase::Enrollment.is_terminal());
        assert!(!Phase::Preparation.is_terminal());
        assert!(!Phase::Assignment.is_terminal());
        assert!(!Phase::Delivery.is_terminal());
    }

    // Story: can_advance() matches try_advance().is_ok()
    #[test]
    fn test_can_advance_matches_try_advance() {
        let all_phases = [
            Phase::Enrollment,
            Phase::Preparation,
            Phase::Assignment,
            Phase::Delivery,
            Phase::Complete,
            Phase::Cancelled,
        ];
        for phase in all_phases {
            assert_eq!(
                phase.can_advance(),
                phase.try_advance().is_ok(),
                "can_advance() must match try_advance().is_ok() for {phase}"
            );
        }
    }

    // Story: Display outputs exact Postgres enum values
    #[test]
    fn test_display_matches_postgres_enum_values() {
        assert_eq!(Phase::Enrollment.to_string(), "enrollment");
        assert_eq!(Phase::Preparation.to_string(), "preparation");
        assert_eq!(Phase::Assignment.to_string(), "assignment");
        assert_eq!(Phase::Delivery.to_string(), "delivery");
        assert_eq!(Phase::Complete.to_string(), "complete");
        assert_eq!(Phase::Cancelled.to_string(), "cancelled");
    }
}

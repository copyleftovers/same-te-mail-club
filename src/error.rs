/// Extract the user-facing message from a `ServerFnError`, stripping
/// the framework-added "error running server function: " prefix.
pub(crate) fn strip_server_error_prefix(e: &leptos::prelude::ServerFnError) -> String {
    let msg = e.to_string();
    msg.strip_prefix("error running server function: ")
        .unwrap_or(&msg)
        .to_string()
}

/// ASCII Unit Separator used to encode a stable field discriminant alongside
/// the localized user-facing message in a single `ServerFnError` string.
///
/// Format: `"<field_key>\u{1f}<localized_message>"` — `field_key` is a
/// server-function-specific stable identifier (e.g. `"city"`, `"np_number"`,
/// `"signup_deadline"`). Infra/DB/auth errors carry no separator and route
/// to the page-level banner instead of a specific form field.
///
/// Shared wire-format contract between every server function that routes a
/// validation error to one specific form field and the client-side parser
/// that reads it back out: `pages::onboarding`, `pages::home`, `admin::season`.
pub(crate) const FIELD_DISCRIMINANT_SEPARATOR: char = '\u{1f}';

#[cfg(feature = "ssr")]
use crate::types::{InvalidTransition, Phase};

#[cfg(feature = "ssr")]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("invalid phase transition from {from}")]
    InvalidTransition {
        from: Phase,
        #[source]
        source: InvalidTransition,
    },

    #[error("rate limited")]
    RateLimited,

    #[error("SMS delivery failed: {0}")]
    SmsFailed(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[cfg(feature = "ssr")]
impl From<InvalidTransition> for AppError {
    fn from(t: InvalidTransition) -> Self {
        Self::InvalidTransition {
            from: t.from,
            source: t,
        }
    }
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::InvalidTransition { .. } => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::SmsFailed(_) => StatusCode::BAD_GATEWAY,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

#[cfg(feature = "ssr")]
impl AppError {
    /// Convert to a `ServerFnError` for use in Leptos server functions.
    /// Cannot be a `From` impl because `ServerFnError` has a blanket
    /// `impl<E: std::error::Error> From<E>` that conflicts.
    pub fn into_server_fn_error(self) -> leptos::prelude::ServerFnError {
        leptos::prelude::ServerFnError::new(self.to_string())
    }
}

/// Shorthand for converting `sqlx::Error` to `ServerFnError` in `.map_err()`.
///
/// Usage: `.map_err(db_err)?` instead of
/// `.map_err(|e| ServerFnError::new(format!("database error: {e}")))?`
#[cfg(feature = "ssr")]
pub fn db_err(e: sqlx::Error) -> leptos::prelude::ServerFnError {
    AppError::from(e).into_server_fn_error()
}

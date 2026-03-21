use crate::types::{InvalidTransition, Phase};

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

impl From<InvalidTransition> for AppError {
    fn from(t: InvalidTransition) -> Self {
        Self::InvalidTransition {
            from: t.from,
            source: t,
        }
    }
}

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
pub fn db_err(e: sqlx::Error) -> leptos::prelude::ServerFnError {
    AppError::from(e).into_server_fn_error()
}

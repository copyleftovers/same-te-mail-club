#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub turbosms_token: String,
    pub turbosms_sender: String,
    pub csrf_secret: u128,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing DATABASE_URL")]
    MissingDatabaseUrl,
    #[error("missing TURBOSMS_TOKEN")]
    MissingTurbosmsToken,
    #[error("missing TURBOSMS_SENDER")]
    MissingTurbosmsSender,
}

impl Config {
    /// Read from environment. Fails fast naming the missing variable.
    ///
    /// When `SAMETE_SMS_DRY_RUN=true`, TURBOSMS credentials are optional
    /// (they are never used in dry-run mode, so requiring them would block
    /// local development and E2E testing without real SMS credentials).
    ///
    /// # Errors
    ///
    /// Returns `Err` if any required environment variable is absent.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;

        let dry_run = std::env::var("SAMETE_SMS_DRY_RUN").as_deref() == Ok("true");

        let turbosms_token = if dry_run {
            std::env::var("TURBOSMS_TOKEN").unwrap_or_default()
        } else {
            std::env::var("TURBOSMS_TOKEN").map_err(|_| ConfigError::MissingTurbosmsToken)?
        };

        let turbosms_sender = if dry_run {
            std::env::var("TURBOSMS_SENDER").unwrap_or_default()
        } else {
            std::env::var("TURBOSMS_SENDER").map_err(|_| ConfigError::MissingTurbosmsSender)?
        };

        let csrf_secret = std::env::var("CSRF_SECRET")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(rand::random::<u128>);

        Ok(Self {
            database_url,
            turbosms_token,
            turbosms_sender,
            csrf_secret,
        })
    }
}

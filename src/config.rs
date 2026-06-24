#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub turbosms_token: String,
    pub turbosms_sender: String,
    pub csrf_secret: u128,
    pub sms_dry_run: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing DATABASE_URL")]
    MissingDatabaseUrl,
    #[error("missing TURBOSMS_TOKEN")]
    MissingTurbosmsToken,
    #[error("missing TURBOSMS_SENDER")]
    MissingTurbosmsSender,
    #[error("TURBOSMS_TOKEN must not be empty")]
    EmptyTurbosmsToken,
    #[error("TURBOSMS_SENDER must not be empty")]
    EmptyTurbosmsSender,
}

impl Config {
    /// Read from environment. Fails fast naming the missing or invalid variable.
    ///
    /// When `SAMETE_SMS_DRY_RUN=true`, `TurboSMS` credentials are optional and
    /// not validated — they are never used in dry-run mode, so requiring them
    /// would block local development and E2E testing without real credentials.
    ///
    /// In production (non-dry-run), both `TURBOSMS_TOKEN` and `TURBOSMS_SENDER`
    /// must be present and non-empty. An empty bearer token would cause a 401
    /// at the first SMS send.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any required environment variable is absent or empty.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;

        let sms_dry_run = std::env::var("SAMETE_SMS_DRY_RUN").as_deref() == Ok("true");

        let turbosms_token = if sms_dry_run {
            std::env::var("TURBOSMS_TOKEN").unwrap_or_default()
        } else {
            let token =
                std::env::var("TURBOSMS_TOKEN").map_err(|_| ConfigError::MissingTurbosmsToken)?;
            if token.is_empty() {
                return Err(ConfigError::EmptyTurbosmsToken);
            }
            token
        };

        let turbosms_sender = if sms_dry_run {
            std::env::var("TURBOSMS_SENDER").unwrap_or_default()
        } else {
            let sender =
                std::env::var("TURBOSMS_SENDER").map_err(|_| ConfigError::MissingTurbosmsSender)?;
            if sender.is_empty() {
                return Err(ConfigError::EmptyTurbosmsSender);
            }
            sender
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
            sms_dry_run,
        })
    }
}

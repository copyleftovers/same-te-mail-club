#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub turbosms_token: String,
    pub turbosms_sender: String,
    // CSRF: SameSite=Strict cookie attribute is the mitigation (see src/pages/login.rs set_cookie_header)
    pub sms_dry_run: bool,
    pub admin_bootstrap: Option<AdminBootstrap>,
}

/// First admin ensured at every boot (idempotent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminBootstrap {
    /// E.164-normalized phone.
    pub phone: String,
    pub name: String,
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
    #[error("SAMETE_ADMIN_PHONE and SAMETE_ADMIN_NAME must be set together")]
    AdminBootstrapIncomplete,
    #[error("SAMETE_ADMIN_PHONE is not a valid Ukrainian phone: {0}")]
    InvalidAdminPhone(String),
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
    /// Returns `Err` if any required environment variable is absent or empty,
    /// if exactly one of `SAMETE_ADMIN_PHONE`/`SAMETE_ADMIN_NAME` is set, or
    /// if the admin phone is invalid.
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

        let admin_bootstrap = admin_bootstrap_from_vars(
            std::env::var("SAMETE_ADMIN_PHONE").ok(),
            std::env::var("SAMETE_ADMIN_NAME").ok(),
        )?;

        Ok(Self {
            database_url,
            turbosms_token,
            turbosms_sender,
            sms_dry_run,
            admin_bootstrap,
        })
    }
}

/// Parse the optional first-admin bootstrap. Blank values count as unset.
fn admin_bootstrap_from_vars(
    phone: Option<String>,
    name: Option<String>,
) -> Result<Option<AdminBootstrap>, ConfigError> {
    let present = |v: Option<String>| v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    match (present(phone), present(name)) {
        (None, None) => Ok(None),
        (Some(phone), Some(name)) => {
            let phone = crate::phone::normalize(&phone)
                .map_err(|_| ConfigError::InvalidAdminPhone(phone.clone()))?;
            Ok(Some(AdminBootstrap { phone, name }))
        }
        _ => Err(ConfigError::AdminBootstrapIncomplete),
    }
}

#[cfg(test)]
mod tests {
    use super::{AdminBootstrap, ConfigError, admin_bootstrap_from_vars};

    #[test]
    fn neither_var_means_no_bootstrap() {
        assert_eq!(admin_bootstrap_from_vars(None, None).unwrap(), None);
    }

    #[test]
    fn blank_vars_count_as_unset() {
        assert_eq!(
            admin_bootstrap_from_vars(Some("  ".into()), Some(String::new())).unwrap(),
            None
        );
    }

    #[test]
    fn both_vars_normalize_phone_and_trim_name() {
        assert_eq!(
            admin_bootstrap_from_vars(Some("067 123 45 67".into()), Some("  Організатор ".into()))
                .unwrap(),
            Some(AdminBootstrap {
                phone: "+380671234567".into(),
                name: "Організатор".into()
            })
        );
    }

    #[test]
    fn phone_without_name_is_refused() {
        assert!(matches!(
            admin_bootstrap_from_vars(Some("+380671234567".into()), None),
            Err(ConfigError::AdminBootstrapIncomplete)
        ));
    }

    #[test]
    fn name_without_phone_is_refused() {
        assert!(matches!(
            admin_bootstrap_from_vars(None, Some("Організатор".into())),
            Err(ConfigError::AdminBootstrapIncomplete)
        ));
    }

    #[test]
    fn invalid_phone_is_refused() {
        assert!(matches!(
            admin_bootstrap_from_vars(Some("12345".into()), Some("A".into())),
            Err(ConfigError::InvalidAdminPhone(_))
        ));
    }
}

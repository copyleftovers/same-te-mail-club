#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub sms: SmsMode,
    pub admin_bootstrap: Option<AdminBootstrap>,
}

/// How SMS is delivered. Test mode exists only inside `DryRun`, so a server
/// that sends real SMS can never run with the fixed test OTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmsMode {
    /// Real `TurboSMS` delivery.
    Live { token: String, sender: String },
    /// SMS is logged, never sent. `test_mode` = fixed OTP `000000`, no OTP
    /// rate limits, no deadline gates (local dev / E2E only).
    DryRun { test_mode: bool },
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
    #[error("SAMETE_TEST_MODE=true requires SAMETE_SMS_DRY_RUN=true")]
    TestModeRequiresDryRun,
    #[error("SAMETE_TEST_MODE=true requires a loopback bind address, got {0}")]
    TestModeRequiresLoopback(std::net::SocketAddr),
    #[error("SAMETE_ADMIN_PHONE and SAMETE_ADMIN_NAME must be set together")]
    AdminBootstrapIncomplete,
    #[error("SAMETE_ADMIN_PHONE is not a valid Ukrainian phone: {0}")]
    InvalidAdminPhone(String),
}

impl Config {
    /// Read from environment. Fails fast naming the missing or invalid variable.
    ///
    /// `SAMETE_SMS_DRY_RUN=true` logs SMS instead of sending them, so
    /// `TurboSMS` credentials are not required. Otherwise `TURBOSMS_TOKEN` and
    /// `TURBOSMS_SENDER` must be present and non-empty (an empty bearer token
    /// would cause a 401 at the first SMS send).
    ///
    /// `SAMETE_TEST_MODE=true` (fixed OTP, no OTP rate limits, no deadline
    /// gates) is refused unless `SAMETE_SMS_DRY_RUN=true`; the loopback-bind
    /// refusal is enforced at boot by [`Config::check_bind_addr`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if any required environment variable is absent or empty,
    /// if exactly one of `SAMETE_ADMIN_PHONE`/`SAMETE_ADMIN_NAME` is set, or
    /// if the admin phone is invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;

        let sms = sms_mode_from_vars(
            std::env::var("SAMETE_SMS_DRY_RUN").ok().as_deref(),
            std::env::var("SAMETE_TEST_MODE").ok().as_deref(),
            std::env::var("TURBOSMS_TOKEN").ok(),
            std::env::var("TURBOSMS_SENDER").ok(),
        )?;

        let admin_bootstrap = admin_bootstrap_from_vars(
            std::env::var("SAMETE_ADMIN_PHONE").ok(),
            std::env::var("SAMETE_ADMIN_NAME").ok(),
        )?;

        Ok(Self {
            database_url,
            sms,
            admin_bootstrap,
        })
    }

    /// True only in dry-run SMS mode with `SAMETE_TEST_MODE=true`.
    #[must_use]
    pub fn test_mode(&self) -> bool {
        matches!(self.sms, SmsMode::DryRun { test_mode: true })
    }

    /// Refuse test mode on any non-loopback bind address.
    ///
    /// # Errors
    ///
    /// Returns `Err(ConfigError::TestModeRequiresLoopback)` when test mode is
    /// on and `addr` is not loopback.
    pub fn check_bind_addr(&self, addr: std::net::SocketAddr) -> Result<(), ConfigError> {
        if self.test_mode() && !addr.ip().is_loopback() {
            return Err(ConfigError::TestModeRequiresLoopback(addr));
        }
        Ok(())
    }
}

/// Parse SMS delivery mode from raw env values. Only the literal `"true"` enables a flag.
fn sms_mode_from_vars(
    dry_run: Option<&str>,
    test_mode: Option<&str>,
    token: Option<String>,
    sender: Option<String>,
) -> Result<SmsMode, ConfigError> {
    let test_mode = test_mode == Some("true");
    if dry_run == Some("true") {
        return Ok(SmsMode::DryRun { test_mode });
    }
    if test_mode {
        return Err(ConfigError::TestModeRequiresDryRun);
    }
    let token = token.ok_or(ConfigError::MissingTurbosmsToken)?;
    if token.is_empty() {
        return Err(ConfigError::EmptyTurbosmsToken);
    }
    let sender = sender.ok_or(ConfigError::MissingTurbosmsSender)?;
    if sender.is_empty() {
        return Err(ConfigError::EmptyTurbosmsSender);
    }
    Ok(SmsMode::Live { token, sender })
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
    use super::{
        AdminBootstrap, Config, ConfigError, SmsMode, admin_bootstrap_from_vars, sms_mode_from_vars,
    };

    fn config_with(sms: SmsMode) -> Config {
        Config {
            database_url: String::new(),
            sms,
            admin_bootstrap: None,
        }
    }

    #[test]
    fn neither_var_means_no_bootstrap() {
        assert_eq!(admin_bootstrap_from_vars(None, None).ok(), Some(None));
    }

    #[test]
    fn blank_vars_count_as_unset() {
        assert_eq!(
            admin_bootstrap_from_vars(Some("  ".into()), Some(String::new())).ok(),
            Some(None)
        );
    }

    #[test]
    fn both_vars_normalize_phone_and_trim_name() {
        assert_eq!(
            admin_bootstrap_from_vars(Some("067 123 45 67".into()), Some("  Організатор ".into()))
                .ok(),
            Some(Some(AdminBootstrap {
                phone: "+380671234567".into(),
                name: "Організатор".into()
            }))
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

    #[test]
    fn dry_run_without_test_mode() {
        assert_eq!(
            sms_mode_from_vars(Some("true"), None, None, None).ok(),
            Some(SmsMode::DryRun { test_mode: false })
        );
    }

    #[test]
    fn dry_run_with_test_mode() {
        assert_eq!(
            sms_mode_from_vars(Some("true"), Some("true"), None, None).ok(),
            Some(SmsMode::DryRun { test_mode: true })
        );
    }

    #[test]
    fn test_mode_without_dry_run_is_refused_even_with_credentials() {
        let result = sms_mode_from_vars(None, Some("true"), Some("tok".into()), Some("snd".into()));
        assert!(matches!(result, Err(ConfigError::TestModeRequiresDryRun)));
    }

    #[test]
    fn live_requires_token() {
        assert!(matches!(
            sms_mode_from_vars(None, None, None, Some("snd".into())),
            Err(ConfigError::MissingTurbosmsToken)
        ));
    }

    #[test]
    fn live_rejects_empty_sender() {
        assert!(matches!(
            sms_mode_from_vars(None, None, Some("tok".into()), Some(String::new())),
            Err(ConfigError::EmptyTurbosmsSender)
        ));
    }

    #[test]
    fn live_with_credentials() {
        assert_eq!(
            sms_mode_from_vars(None, None, Some("tok".into()), Some("snd".into())).ok(),
            Some(SmsMode::Live {
                token: "tok".into(),
                sender: "snd".into()
            })
        );
    }

    #[test]
    fn only_literal_true_enables_flags() {
        assert!(matches!(
            sms_mode_from_vars(Some("1"), Some("TRUE"), None, None),
            Err(ConfigError::MissingTurbosmsToken)
        ));
    }

    #[test]
    fn test_mode_refuses_non_loopback_bind() {
        let config = config_with(SmsMode::DryRun { test_mode: true });
        assert!(matches!(
            config.check_bind_addr("0.0.0.0:3000".parse().expect("addr")),
            Err(ConfigError::TestModeRequiresLoopback(_))
        ));
        assert!(
            config
                .check_bind_addr("127.0.0.1:3000".parse().expect("addr"))
                .is_ok()
        );
    }

    #[test]
    fn non_test_mode_allows_any_bind() {
        let config = config_with(SmsMode::DryRun { test_mode: false });
        assert!(
            config
                .check_bind_addr("0.0.0.0:3000".parse().expect("addr"))
                .is_ok()
        );
    }
}

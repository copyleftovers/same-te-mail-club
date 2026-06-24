use std::time::Duration;

use crate::config::Config;

const TURBOSMS_SEND_URL: &str = "https://api.turbosms.ua/message/send.json";
const RETRY_DELAY: Duration = Duration::from_secs(1);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Build a shared HTTP client for SMS delivery.
///
/// Called once at startup. The returned client enforces a 10-second timeout
/// per request and reuses connections across all SMS calls.
///
/// # Errors
///
/// Returns `Err` if the TLS backend or client builder fails.
pub fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::ClientBuilder::new().timeout(HTTP_TIMEOUT).build()
}

/// Send an SMS via the `TurboSMS` API.
///
/// When `config.sms_dry_run` is true, the message is logged instead of sent.
///
/// Uses the shared `client` for connection reuse and enforced timeout.
/// On transient errors (HTTP 5xx or network failure), retries once after
/// one second. Client errors (4xx) are not retried.
///
/// # Errors
///
/// Returns `Err` on HTTP transport error, non-success API response,
/// missing `response_result` in the `TurboSMS` payload, or a non-zero
/// `response_code` in any result entry.
pub async fn send_sms(
    config: &Config,
    client: &reqwest::Client,
    phone: &str,
    message: &str,
) -> Result<(), SmsError> {
    if config.sms_dry_run {
        tracing::info!(
            phone = phone,
            message = message,
            "[DRY RUN] SMS would be sent"
        );
        return Ok(());
    }

    let body = serde_json::json!({
        "recipients": [phone],
        "sms": {
            "sender": config.turbosms_sender,
            "text": message,
        }
    });

    match post_to_turbosms(client, config, &body).await {
        Ok(()) => Ok(()),
        Err(e) if is_transient(&e) => {
            tracing::warn!(phone = phone, error = %e, "transient SMS error, retrying in 1s");
            tokio::time::sleep(RETRY_DELAY).await;
            post_to_turbosms(client, config, &body).await
        }
        Err(e) => Err(e),
    }
}

/// Execute a single POST to `TurboSMS` and validate the response.
async fn post_to_turbosms(
    client: &reqwest::Client,
    config: &Config,
    body: &serde_json::Value,
) -> Result<(), SmsError> {
    let response = client
        .post(TURBOSMS_SEND_URL)
        .bearer_auth(&config.turbosms_token)
        .json(body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(if status.is_server_error() {
            SmsError::TransientApiError(format!("HTTP {status}: {text}"))
        } else {
            SmsError::ApiError(format!("HTTP {status}: {text}"))
        });
    }

    validate_turbosms_response(response).await
}

/// Parse the `TurboSMS` JSON response and ensure every recipient succeeded.
///
/// Requires `response_result` to be present as an array.
/// Each entry must have `response_code == 0`; any non-zero code or
/// missing `response_code` is treated as an error.
async fn validate_turbosms_response(response: reqwest::Response) -> Result<(), SmsError> {
    let body = response.text().await?;
    tracing::debug!(raw_body = %body, "TurboSMS response body");

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let results = json
        .get("response_result")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            SmsError::ApiError("missing response_result in TurboSMS response".to_owned())
        })?;

    for result in results {
        let code = result
            .get("response_code")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                SmsError::ApiError("missing response_code in TurboSMS result entry".to_owned())
            })?;

        if code != 0 {
            let msg = result
                .get("response_status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown error")
                .to_owned();
            return Err(SmsError::ApiError(msg));
        }
    }

    Ok(())
}

/// Whether an error is transient and worth retrying.
fn is_transient(error: &SmsError) -> bool {
    match error {
        SmsError::HttpError(e) => e.is_timeout() || e.is_connect(),
        SmsError::TransientApiError(_) => true,
        SmsError::ApiError(_) | SmsError::JsonError(_) => false,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SmsError {
    #[error("TurboSMS API error: {0}")]
    ApiError(String),
    #[error("TurboSMS transient error: {0}")]
    TransientApiError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

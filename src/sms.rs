use crate::config::Config;

/// Send an SMS via the `TurboSMS` API.
/// `POST https://api.turbosms.ua/message/send.json`
/// Body: `{ "recipients": [phone], "sms": { "sender": alpha_name, "text": message } }`
/// Bearer token auth.
///
/// When `SAMETE_SMS_DRY_RUN=true` env var is set, the message is logged instead of sent.
///
/// # Errors
///
/// Returns `Err` on HTTP transport error or non-success API response.
pub async fn send_sms(config: &Config, phone: &str, message: &str) -> Result<(), SmsError> {
    if std::env::var("SAMETE_SMS_DRY_RUN").as_deref() == Ok("true") {
        tracing::info!(
            phone = phone,
            message = message,
            "[DRY RUN] SMS would be sent"
        );
        return Ok(());
    }

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "recipients": [phone],
        "sms": {
            "sender": config.turbosms_sender,
            "text": message,
        }
    });

    let response = client
        .post("https://api.turbosms.ua/message/send.json")
        .bearer_auth(&config.turbosms_token)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(SmsError::ApiError(format!("HTTP {status}: {text}")));
    }

    // TurboSMS returns a JSON body with a "response_result" array.
    // Each element has a "responseCode" field; 0 = success.
    let json: serde_json::Value = response.json().await?;
    if let Some(results) = json
        .get("response_result")
        .and_then(serde_json::Value::as_array)
    {
        for result in results {
            if let Some(code) = result
                .get("responseCode")
                .and_then(serde_json::Value::as_i64)
                && code != 0
            {
                let msg = result
                    .get("responseMessage")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown error")
                    .to_owned();
                return Err(SmsError::ApiError(msg));
            }
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SmsError {
    #[error("TurboSMS API error: {0}")]
    ApiError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

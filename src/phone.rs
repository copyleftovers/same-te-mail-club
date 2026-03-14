use phonenumber::country;

/// Normalize a phone input to E.164 (`+380XXXXXXXXX`).
///
/// Strips whitespace, hyphens, parentheses, dots.
/// Replaces leading 0 with +380, prepends + if starts with 380.
/// Validates with `phonenumber` crate (default region UA).
///
/// # Errors
///
/// Returns `Err(InvalidFormat)` for structurally invalid numbers and
/// `Err(NotUkrainian)` for valid numbers outside the +380 country code.
pub fn normalize(raw: &str) -> Result<String, PhoneError> {
    // Strip whitespace, hyphens, parentheses, dots
    let cleaned: String = raw
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '(' | ')' | '.'))
        .collect();

    if cleaned.is_empty() {
        return Err(PhoneError::InvalidFormat);
    }

    // Normalize prefix: leading 0 → +380, leading 380 → +380
    let normalized = if let Some(rest) = cleaned.strip_prefix('0') {
        format!("+380{rest}")
    } else if cleaned.starts_with("380") {
        format!("+{cleaned}")
    } else {
        cleaned
    };

    // Parse with phonenumber crate, hint region UA
    let parsed = phonenumber::parse(Some(country::UA), &normalized)
        .map_err(|_| PhoneError::InvalidFormat)?;

    // Country code must be 380 (Ukraine) — checked before validity so that
    // non-Ukrainian numbers get NotUkrainian rather than InvalidFormat.
    if parsed.code().value() != 380 {
        return Err(PhoneError::NotUkrainian);
    }

    // Validate: must be a structurally valid Ukrainian number
    if !phonenumber::is_valid(&parsed) {
        return Err(PhoneError::InvalidFormat);
    }

    // Format as E.164
    Ok(phonenumber::format(&parsed).to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("invalid phone number format")]
    InvalidFormat,
    #[error("only Ukrainian (+380) numbers are supported")]
    NotUkrainian,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_leading_zero() {
        assert_eq!(normalize("0671234567").unwrap(), "+380671234567");
    }

    #[test]
    fn test_normalize_already_e164() {
        assert_eq!(normalize("+380671234567").unwrap(), "+380671234567");
    }

    #[test]
    fn test_normalize_380_prefix() {
        assert_eq!(normalize("380671234567").unwrap(), "+380671234567");
    }

    #[test]
    fn test_normalize_hyphens() {
        assert_eq!(normalize("067-123-45-67").unwrap(), "+380671234567");
    }

    #[test]
    fn test_normalize_parens_spaces() {
        assert_eq!(normalize("(067) 123 45 67").unwrap(), "+380671234567");
    }

    #[test]
    fn test_non_ukrainian_number() {
        assert!(matches!(
            normalize("+1234567890"),
            Err(PhoneError::NotUkrainian)
        ));
    }

    #[test]
    fn test_invalid_format_alpha() {
        assert!(matches!(normalize("abc"), Err(PhoneError::InvalidFormat)));
    }

    #[test]
    fn test_invalid_format_empty() {
        assert!(matches!(normalize(""), Err(PhoneError::InvalidFormat)));
    }
}

/// Parsed representation of a single key from the `key_internal_data` attribute.
///
/// Raw format: `{key_id}: {status} {algorithm} {counter}`
/// Example: `57850d1d41fd: valid jws_es256 0`
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedKey {
    pub id: String,
    pub status: KeyStatus,
    pub algorithm: String,
    pub counter: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyStatus {
    Valid,
    Retired,
    Revoked,
    Unknown,
}

impl KeyStatus {
    pub fn label(self) -> &'static str {
        match self {
            KeyStatus::Valid => "Valid",
            KeyStatus::Retired => "Retired",
            KeyStatus::Revoked => "Revoked",
            KeyStatus::Unknown => "Unknown",
        }
    }

    pub fn badge_classes(self) -> &'static str {
        match self {
            KeyStatus::Valid => "bg-success-soft text-success",
            KeyStatus::Retired => "bg-warning-soft text-warning",
            KeyStatus::Revoked => "bg-danger-soft text-danger",
            KeyStatus::Unknown => "bg-elevated text-tertiary",
        }
    }

    /// Sort order: Valid < Retired < Revoked < Unknown
    pub(crate) fn sort_order(self) -> u8 {
        match self {
            KeyStatus::Valid => 0,
            KeyStatus::Retired => 1,
            KeyStatus::Revoked => 2,
            KeyStatus::Unknown => 3,
        }
    }
}

/// Parse a single `key_internal_data` value into a [`ParsedKey`].
///
/// Returns `None` if the value does not match the expected format.
pub fn parse_key_state(value: &str) -> Option<ParsedKey> {
    // Step 1: split on first ':'.
    let colon = value.find(':')?;
    let id = value[..colon].trim().to_string();
    let rest = value[colon + 1..].trim();

    // Step 2: split right side on whitespace — expect exactly 3 tokens.
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let status_str = parts[0];
    let algorithm = parts[1].to_string();
    let counter_str = parts[2];

    // Step 3: parse status (case-insensitive).
    let status = match status_str.to_ascii_lowercase().as_str() {
        "valid" => KeyStatus::Valid,
        "retired" => KeyStatus::Retired,
        "revoked" => KeyStatus::Revoked,
        _ => KeyStatus::Unknown,
    };

    // Step 4: parse counter.
    let counter: u64 = counter_str.parse().ok()?;

    Some(ParsedKey { id, status, algorithm, counter })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // --- Live fixtures (4 distinct from the spec) ---

    #[test]
    fn live_fixture_jws_es256_a() {
        let parsed = parse_key_state("57850d1d41fd: valid jws_es256 0").unwrap();
        assert_eq!(parsed.id, "57850d1d41fd");
        assert_eq!(parsed.status, KeyStatus::Valid);
        assert_eq!(parsed.algorithm, "jws_es256");
        assert_eq!(parsed.counter, 0);
    }

    #[test]
    fn live_fixture_jwe_a128gcm() {
        let parsed = parse_key_state("95666a75afa2: valid jwe_a128gcm 0").unwrap();
        assert_eq!(parsed.id, "95666a75afa2");
        assert_eq!(parsed.status, KeyStatus::Valid);
        assert_eq!(parsed.algorithm, "jwe_a128gcm");
        assert_eq!(parsed.counter, 0);
    }

    #[test]
    fn live_fixture_jws_rs256() {
        let parsed = parse_key_state("c73bff25ebd2: valid jws_rs256 0").unwrap();
        assert_eq!(parsed.id, "c73bff25ebd2");
        assert_eq!(parsed.status, KeyStatus::Valid);
        assert_eq!(parsed.algorithm, "jws_rs256");
        assert_eq!(parsed.counter, 0);
    }

    #[test]
    fn live_fixture_second_jwe() {
        let parsed = parse_key_state("d90cedd0d90f: valid jws_es256 0").unwrap();
        assert_eq!(parsed.id, "d90cedd0d90f");
        assert_eq!(parsed.status, KeyStatus::Valid);
        assert_eq!(parsed.algorithm, "jws_es256");
        assert_eq!(parsed.counter, 0);
    }

    // --- Malformed inputs ---

    #[test]
    fn malformed_no_colon_returns_none() {
        assert_eq!(parse_key_state("57850d1d41fd valid jws_es256 0"), None);
    }

    #[test]
    fn malformed_only_two_right_side_fields_returns_none() {
        assert_eq!(parse_key_state("57850d1d41fd: valid jws_es256"), None);
    }

    #[test]
    fn malformed_bad_counter_returns_none() {
        assert_eq!(parse_key_state("57850d1d41fd: valid jws_es256 not_a_number"), None);
    }

    #[test]
    fn malformed_empty_string_returns_none() {
        assert_eq!(parse_key_state(""), None);
    }

    // --- Unknown status ---

    #[test]
    fn unknown_status_returns_key_not_none() {
        let parsed = parse_key_state("abc: weird jws_es256 0").unwrap();
        assert_eq!(parsed.id, "abc");
        assert_eq!(parsed.status, KeyStatus::Unknown);
        assert_eq!(parsed.algorithm, "jws_es256");
    }

    // --- Retired / Revoked status ---

    #[test]
    fn retired_status_parsed() {
        let parsed = parse_key_state("aabbcc112233: retired jws_es256 1").unwrap();
        assert_eq!(parsed.status, KeyStatus::Retired);
        assert_eq!(parsed.counter, 1);
    }

    #[test]
    fn revoked_status_parsed() {
        let parsed = parse_key_state("ddeeff445566: revoked jws_rs256 2").unwrap();
        assert_eq!(parsed.status, KeyStatus::Revoked);
        assert_eq!(parsed.counter, 2);
    }

    // --- Label / badge_classes ---

    #[test]
    fn label_values() {
        assert_eq!(KeyStatus::Valid.label(), "Valid");
        assert_eq!(KeyStatus::Retired.label(), "Retired");
        assert_eq!(KeyStatus::Revoked.label(), "Revoked");
        assert_eq!(KeyStatus::Unknown.label(), "Unknown");
    }

    #[test]
    fn badge_class_values() {
        assert!(KeyStatus::Valid.badge_classes().contains("success"));
        assert!(KeyStatus::Retired.badge_classes().contains("warning"));
        assert!(KeyStatus::Revoked.badge_classes().contains("danger"));
        assert!(KeyStatus::Unknown.badge_classes().contains("elevated"));
    }

    // --- Sort order ---

    #[test]
    fn sort_order_valid_before_retired() {
        assert!(KeyStatus::Valid.sort_order() < KeyStatus::Retired.sort_order());
    }

    #[test]
    fn sort_order_retired_before_revoked() {
        assert!(KeyStatus::Retired.sort_order() < KeyStatus::Revoked.sort_order());
    }
}

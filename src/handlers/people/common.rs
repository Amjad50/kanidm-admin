use kanidm_proto::internal::{CredentialDetailType, CredentialStatus};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::kanidm::entry::{attr_all, attr_first, attr_present};
use crate::AppState;
use crate::auth::AdminUser;

// ── Domain helper ─────────────────────────────────────────────────────────────

/// Lazily fetch the domain name from kanidm for the @suffix display.
/// Returns None on any failure rather than propagating an error — the suffix
/// is cosmetic and the form works fine without it.
pub(super) async fn fetch_domain_name(state: &AppState, user: &AdminUser) -> Option<String> {
    let client = state.kanidm.for_token(&user.token).await.ok()?;
    let entry = client.idm_domain_get().await.ok()?;
    attr_first(&entry, "name")
}

// ── Validation helpers ────────────────────────────────────────────────────────

pub fn validate_name(s: &str) -> Result<(), &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Username is required.");
    }
    if s.len() > 63 {
        return Err("Username must be 63 characters or less.");
    }
    if !s.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
        return Err("Username must start with a lowercase letter.");
    }
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-')) {
        return Err("Username may only contain lowercase letters, digits, '.', '_', '-'.");
    }
    Ok(())
}

pub fn validate_displayname(s: &str) -> Result<(), &'static str> {
    if s.trim().is_empty() {
        return Err("Display name is required.");
    }
    Ok(())
}

pub(crate) fn validate_legalname_optional(s: &str) -> Result<(), &'static str> {
    if s.trim().len() > 255 {
        return Err("Legal name must be 255 characters or less.");
    }
    Ok(())
}

pub(crate) fn validate_email_list_optional(emails: &[String]) -> Result<(), &'static str> {
    for e in emails {
        let trimmed = e.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.contains('@') {
            return Err("All emails must contain '@'.");
        }
        if trimmed.len() > 320 {
            return Err("Email addresses must be 320 characters or less.");
        }
    }
    Ok(())
}

pub use crate::handlers::common::friendly_client_error;

// ── Status ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonStatus {
    Active,
    Expired,
    NotYetActive,
}

impl PersonStatus {
    pub fn label(self) -> &'static str {
        match self {
            PersonStatus::Active => "Active",
            PersonStatus::Expired => "Expired",
            PersonStatus::NotYetActive => "Not yet active",
        }
    }

    pub fn badge_classes(self) -> &'static str {
        match self {
            PersonStatus::Active => "bg-success-soft text-success",
            PersonStatus::Expired => "bg-danger-soft text-danger",
            PersonStatus::NotYetActive => "bg-warning-soft text-warning",
        }
    }

    pub fn dot_classes(self) -> &'static str {
        match self {
            PersonStatus::Active => "bg-success",
            PersonStatus::Expired => "bg-danger",
            PersonStatus::NotYetActive => "bg-warning",
        }
    }
}

// ── Shared credential summary ─────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum PrimaryCred {
    None,
    Password,
    PasswordWithTotp,
    PasswordWithBackupCode,
    GeneratedPassword,
    Other,
}

impl PrimaryCred {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            PrimaryCred::None => None,
            PrimaryCred::Password => Some("Password"),
            PrimaryCred::PasswordWithTotp => Some("Password + TOTP"),
            PrimaryCred::PasswordWithBackupCode => Some("Password + Backup codes"),
            PrimaryCred::GeneratedPassword => Some("Generated password"),
            PrimaryCred::Other => Some("Configured"),
        }
    }
}

pub struct CredentialSummary {
    pub primary: PrimaryCred,
    pub passkey_count: usize,
    pub passkey_names: Vec<String>,
    pub attested_passkey_count: usize,
    pub attested_passkey_names: Vec<String>,
    pub ssh_key_count: usize,
    pub backup_codes_remaining: Option<usize>,
    pub totp_labels: Vec<String>,
    pub radius_configured: bool,
}

/// Build a `CredentialSummary` from an entry and optional live credential status.
/// Pass `status = None` when no API call has been made (e.g. the Overview tab).
pub fn summarize_credentials(
    entry: &kanidm_proto::v1::Entry,
    status: Option<&CredentialStatus>,
) -> CredentialSummary {
    let passkey_names = attr_all(entry, "passkeys");
    let passkey_count = passkey_names.len();
    let attested_passkey_names = attr_all(entry, "attested_passkeys");
    let attested_passkey_count = attested_passkey_names.len();
    let ssh_key_count = attr_all(entry, "ssh_publickey").len();
    let radius_configured = attr_present(entry, "radius_secret");

    let (primary, totp_labels, backup_codes_remaining) =
        if let Some(st) = status {
            let mut primary = PrimaryCred::None;
            let mut totp_labels: Vec<String> = vec![];
            let mut backup_codes_remaining: Option<usize> = None;

            for cred in &st.creds {
                match &cred.type_ {
                    CredentialDetailType::Password => {
                        primary = PrimaryCred::Password;
                    }
                    CredentialDetailType::GeneratedPassword => {
                        primary = PrimaryCred::GeneratedPassword;
                    }
                    CredentialDetailType::PasswordMfa(totps, _wan_labels, count) => {
                        if !totps.is_empty() {
                            primary = PrimaryCred::PasswordWithTotp;
                            totp_labels = totps.clone();
                        } else if *count > 0 {
                            primary = PrimaryCred::PasswordWithBackupCode;
                        } else {
                            primary = PrimaryCred::Other;
                        }
                        if *count > 0 {
                            backup_codes_remaining = Some(*count);
                        }
                    }
                    CredentialDetailType::Passkey(_) => {}
                }
            }

            (primary, totp_labels, backup_codes_remaining)
        } else {
            let primary = if attr_present(entry, "primary_credential") {
                PrimaryCred::Password
            } else {
                PrimaryCred::None
            };
            (primary, vec![], None)
        };

    CredentialSummary {
        primary,
        passkey_count,
        passkey_names,
        attested_passkey_count,
        attested_passkey_names,
        ssh_key_count,
        backup_codes_remaining,
        totp_labels,
        radius_configured,
    }
}

pub(super) fn parse_kanidm_datetime(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s, &Rfc3339).ok()
}

pub(super) fn compute_status_at(entry: &kanidm_proto::v1::Entry, now: OffsetDateTime) -> PersonStatus {
    let valid_from = attr_first(entry, "account_valid_from")
        .as_deref()
        .and_then(parse_kanidm_datetime);

    let expire = attr_first(entry, "account_expire")
        .as_deref()
        .and_then(parse_kanidm_datetime);

    if let Some(exp) = expire
        && exp <= now {
            return PersonStatus::Expired;
        }

    if let Some(vf) = valid_from
        && vf > now {
            return PersonStatus::NotYetActive;
        }

    PersonStatus::Active
}

#[cfg(test)]
mod credential_summary_tests {
    use std::collections::BTreeMap;
    use kanidm_proto::internal::{CredentialDetail, CredentialDetailType, CredentialStatus};
    use uuid::Uuid;

    use super::{summarize_credentials, PrimaryCred};

    fn entry(attrs: &[(&str, &[&str])]) -> kanidm_proto::v1::Entry {
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (k, values) in attrs {
            for v in *values {
                map.entry(k.to_string()).or_default().push(v.to_string());
            }
        }
        kanidm_proto::v1::Entry { attrs: map }
    }

    #[test]
    fn passkey_only_entry_no_status() {
        let e = entry(&[("passkeys", &["bitwarden-new"])]);
        let s = summarize_credentials(&e, None);
        assert_eq!(s.primary, PrimaryCred::None);
        assert_eq!(s.passkey_count, 1);
        assert_eq!(s.passkey_names, vec!["bitwarden-new".to_string()]);
        assert_eq!(s.attested_passkey_count, 0);
    }

    #[test]
    fn empty_entry_gives_all_zeros() {
        let e = entry(&[]);
        let s = summarize_credentials(&e, None);
        assert_eq!(s.primary, PrimaryCred::None);
        assert_eq!(s.passkey_count, 0);
        assert_eq!(s.attested_passkey_count, 0);
        assert_eq!(s.ssh_key_count, 0);
        assert!(!s.radius_configured);
        assert!(s.totp_labels.is_empty());
        assert!(s.backup_codes_remaining.is_none());
    }

    #[test]
    fn password_status_maps_to_primary_password() {
        let e = entry(&[("primary_credential", &["primary"])]);
        let status = CredentialStatus {
            creds: vec![CredentialDetail {
                uuid: Uuid::nil(),
                type_: CredentialDetailType::Password,
            }],
        };
        let s = summarize_credentials(&e, Some(&status));
        assert_eq!(s.primary, PrimaryCred::Password);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use time::macros::datetime;

    use super::{compute_status_at, parse_kanidm_datetime, PersonStatus};

    fn entry(attrs: &[(&str, &str)]) -> kanidm_proto::v1::Entry {
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (k, v) in attrs {
            map.entry(k.to_string()).or_default().push(v.to_string());
        }
        kanidm_proto::v1::Entry { attrs: map }
    }

    #[test]
    fn status_active_when_no_validity_attrs() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let e = entry(&[]);
        assert_eq!(compute_status_at(&e, now), PersonStatus::Active);
    }

    #[test]
    fn status_expired_when_account_expire_in_past() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let e = entry(&[("account_expire", "2026-05-14T00:00:00Z")]);
        assert_eq!(compute_status_at(&e, now), PersonStatus::Expired);
    }

    #[test]
    fn status_expired_when_account_expire_equals_now() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let e = entry(&[("account_expire", "2026-05-15T12:00:00Z")]);
        assert_eq!(compute_status_at(&e, now), PersonStatus::Expired);
    }

    #[test]
    fn status_not_yet_active_when_valid_from_in_future() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let e = entry(&[("account_valid_from", "2026-06-01T00:00:00Z")]);
        assert_eq!(compute_status_at(&e, now), PersonStatus::NotYetActive);
    }

    #[test]
    fn status_active_when_valid_from_in_past_and_no_expire() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let e = entry(&[("account_valid_from", "2026-01-01T00:00:00Z")]);
        assert_eq!(compute_status_at(&e, now), PersonStatus::Active);
    }

    #[test]
    fn status_active_when_valid_from_in_past_and_expire_in_future() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let e = entry(&[
            ("account_valid_from", "2026-01-01T00:00:00Z"),
            ("account_expire", "2027-01-01T00:00:00Z"),
        ]);
        assert_eq!(compute_status_at(&e, now), PersonStatus::Active);
    }

    #[test]
    fn parse_rfc3339_basic() {
        let dt = parse_kanidm_datetime("2023-11-28T04:57:55Z");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month() as u8, 11);
        assert_eq!(dt.day(), 28);
    }

    #[test]
    fn parse_rfc3339_with_offset() {
        let dt = parse_kanidm_datetime("2023-11-28T04:57:55+05:30");
        assert!(dt.is_some());
    }

    #[test]
    fn parse_rfc3339_invalid_returns_none() {
        assert!(parse_kanidm_datetime("1700000000").is_none());
        assert!(parse_kanidm_datetime("not-a-date").is_none());
    }
}

#[cfg(test)]
mod optional_validator_tests {
    use super::{validate_email_list_optional, validate_legalname_optional};

    #[test]
    fn legalname_empty_is_ok() {
        assert!(validate_legalname_optional("").is_ok());
    }

    #[test]
    fn legalname_whitespace_only_is_ok() {
        assert!(validate_legalname_optional("   ").is_ok());
    }

    #[test]
    fn legalname_exactly_255_chars_is_ok() {
        assert!(validate_legalname_optional(&"a".repeat(255)).is_ok());
    }

    #[test]
    fn legalname_256_chars_is_rejected() {
        assert!(validate_legalname_optional(&"a".repeat(256)).is_err());
    }

    #[test]
    fn email_list_empty_is_ok() {
        assert!(validate_email_list_optional(&[]).is_ok());
    }

    #[test]
    fn email_list_all_empty_strings_is_ok() {
        let emails = vec!["".to_string(), "  ".to_string()];
        assert!(validate_email_list_optional(&emails).is_ok());
    }

    #[test]
    fn email_list_valid_address_is_ok() {
        let emails = vec!["user@example.com".to_string()];
        assert!(validate_email_list_optional(&emails).is_ok());
    }

    #[test]
    fn email_list_missing_at_is_rejected() {
        let emails = vec!["notanemail".to_string()];
        assert!(validate_email_list_optional(&emails).is_err());
    }

    #[test]
    fn email_list_over_320_chars_is_rejected() {
        let long = format!("{}@example.com", "a".repeat(310));
        assert!(validate_email_list_optional(&[long]).is_err());
    }

    #[test]
    fn email_list_skips_empty_entries_between_valid() {
        let emails = vec!["a@b.com".to_string(), "".to_string(), "c@d.com".to_string()];
        assert!(validate_email_list_optional(&emails).is_ok());
    }
}

#[cfg(test)]
mod validation_tests {
    use super::{validate_displayname, validate_name};

    // ── validate_name ──────────────────────────────────────────────────

    #[test]
    fn name_empty_string_is_rejected() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn name_whitespace_only_is_rejected() {
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn name_starts_with_digit_is_rejected() {
        assert!(validate_name("1user").is_err());
    }

    #[test]
    fn name_starts_with_uppercase_is_rejected() {
        assert!(validate_name("Alice").is_err());
    }

    #[test]
    fn name_starts_with_dot_is_rejected() {
        assert!(validate_name(".hidden").is_err());
    }

    #[test]
    fn name_contains_uppercase_is_rejected() {
        assert!(validate_name("janeD").is_err());
    }

    #[test]
    fn name_contains_space_is_rejected() {
        assert!(validate_name("jane doe").is_err());
    }

    #[test]
    fn name_contains_at_sign_is_rejected() {
        assert!(validate_name("jane@example").is_err());
    }

    #[test]
    fn name_over_63_chars_is_rejected() {
        let long = "a".repeat(64);
        assert!(validate_name(&long).is_err());
    }

    #[test]
    fn name_exactly_63_chars_is_accepted() {
        let exactly63 = "a".repeat(63);
        assert!(validate_name(&exactly63).is_ok());
    }

    #[test]
    fn name_simple_lowercase_is_accepted() {
        assert!(validate_name("alice").is_ok());
    }

    #[test]
    fn name_with_dot_is_accepted() {
        assert!(validate_name("jane.doe").is_ok());
    }

    #[test]
    fn name_with_hyphen_is_accepted() {
        assert!(validate_name("jane-doe").is_ok());
    }

    #[test]
    fn name_with_underscore_is_accepted() {
        assert!(validate_name("service_account").is_ok());
    }

    #[test]
    fn name_with_digits_is_accepted() {
        assert!(validate_name("user42").is_ok());
    }

    #[test]
    fn name_with_leading_trailing_whitespace_trimmed() {
        assert!(validate_name("  alice  ").is_ok());
    }

    // ── validate_displayname ───────────────────────────────────────────

    #[test]
    fn displayname_empty_is_rejected() {
        assert!(validate_displayname("").is_err());
    }

    #[test]
    fn displayname_whitespace_only_is_rejected() {
        assert!(validate_displayname("   ").is_err());
    }

    #[test]
    fn displayname_non_empty_is_accepted() {
        assert!(validate_displayname("Jane Doe").is_ok());
    }

    #[test]
    fn displayname_single_char_is_accepted() {
        assert!(validate_displayname("J").is_ok());
    }

    #[test]
    fn displayname_utf8_is_accepted() {
        assert!(validate_displayname("Ångström Björk").is_ok());
    }

    #[test]
    fn displayname_with_leading_whitespace_trimmed() {
        assert!(validate_displayname("  Alice  ").is_ok());
    }
}

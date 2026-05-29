use crate::kanidm::entry::{attr_all, attr_first};

// ── OAuth2Kind (runtime kind from stored entry) ───────────────────────────────

/// The runtime OAuth2 client kind derived from the entry's class attributes.
pub enum OAuth2Kind {
    Basic,
    Public,
}

impl OAuth2Kind {
    /// Short lowercase label used on list cards: "basic" / "public".
    pub fn as_str_label(&self) -> &'static str {
        match self {
            OAuth2Kind::Basic => "basic",
            OAuth2Kind::Public => "public",
        }
    }

    /// Full human-readable label used on the detail header.
    pub fn full_label(&self) -> &'static str {
        match self {
            OAuth2Kind::Basic => "Basic (confidential)",
            OAuth2Kind::Public => "Public (PKCE-only)",
        }
    }

    pub fn badge_classes(&self) -> &'static str {
        match self {
            OAuth2Kind::Basic => "bg-info-soft text-info",
            OAuth2Kind::Public => "bg-accent-soft text-accent",
        }
    }
}

/// Detect the OAuth2 client kind from a Kanidm entry's class attributes.
pub fn detect_kind(entry: &kanidm_proto::v1::Entry) -> OAuth2Kind {
    let classes = attr_all(entry, "class");
    if classes.iter().any(|c| c == "oauth2_resource_server_basic") {
        return OAuth2Kind::Basic;
    }
    if classes.iter().any(|c| c == "oauth2_resource_server_public") {
        return OAuth2Kind::Public;
    }
    tracing::warn!(
        name = %attr_first(entry, "name").unwrap_or_default(),
        "OAuth2 entry has neither oauth2_resource_server_basic nor oauth2_resource_server_public class; defaulting to Basic"
    );
    OAuth2Kind::Basic
}

// ── OAuth2CreateKind ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, serde::Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OAuth2CreateKind {
    Basic,
    Public,
}

impl OAuth2CreateKind {
    pub fn label(self) -> &'static str {
        match self {
            OAuth2CreateKind::Basic => "Basic (confidential)",
            OAuth2CreateKind::Public => "Public (PKCE-only)",
        }
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

/// OAuth2 client names follow the same lowercase+allowed-chars rules as person
/// usernames, but must start with a lowercase letter (not underscore).
/// Max length: 63 chars (matches kanidm name attr cap).
pub fn validate_oauth2_name(s: &str) -> Result<(), &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Client name is required.");
    }
    if s.len() > 63 {
        return Err("Client name must be 63 characters or less.");
    }
    if !s.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Err("Client name must start with a lowercase letter.");
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
    {
        return Err("Client name may only contain lowercase letters, digits, '.', '_', '-'.");
    }
    Ok(())
}

/// Display name must be non-empty after trimming.
pub fn validate_oauth2_displayname(s: &str) -> Result<(), &'static str> {
    if s.trim().is_empty() {
        return Err("Display name is required.");
    }
    Ok(())
}

/// Landing URL must parse successfully via `url::Url::parse`. Any scheme is
/// accepted (https, http, oc://, myapp://, etc.) — kanidm does not restrict
/// the scheme, and mobile apps legitimately use custom schemes.
pub fn validate_landing_url(s: &str) -> Result<(), &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Landing URL is required.");
    }
    url::Url::parse(s).map_err(|_| "Landing URL is not a valid URL.")?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod name_tests {
    use super::validate_oauth2_name;

    #[test]
    fn empty_is_rejected() {
        assert!(validate_oauth2_name("").is_err());
    }

    #[test]
    fn whitespace_only_is_rejected() {
        assert!(validate_oauth2_name("   ").is_err());
    }

    #[test]
    fn starts_with_uppercase_is_rejected() {
        assert!(validate_oauth2_name("Grafana").is_err());
    }

    #[test]
    fn starts_with_digit_is_rejected() {
        assert!(validate_oauth2_name("1grafana").is_err());
    }

    #[test]
    fn starts_with_dot_is_rejected() {
        assert!(validate_oauth2_name(".grafana").is_err());
    }

    #[test]
    fn uppercase_in_middle_is_rejected() {
        assert!(validate_oauth2_name("grafAna").is_err());
    }

    #[test]
    fn at_sign_is_rejected() {
        assert!(validate_oauth2_name("gra@fana").is_err());
    }

    #[test]
    fn space_is_rejected() {
        assert!(validate_oauth2_name("gra fana").is_err());
    }

    #[test]
    fn over_63_chars_is_rejected() {
        assert!(validate_oauth2_name(&"a".repeat(64)).is_err());
    }

    #[test]
    fn simple_lowercase_is_accepted() {
        assert!(validate_oauth2_name("grafana").is_ok());
    }

    #[test]
    fn with_hyphen_and_dot_is_accepted() {
        assert!(validate_oauth2_name("my-app.prod").is_ok());
    }

    #[test]
    fn with_underscore_is_accepted() {
        assert!(validate_oauth2_name("my_oauth2_client").is_ok());
    }

    #[test]
    fn with_digits_is_accepted() {
        assert!(validate_oauth2_name("app42").is_ok());
    }

    #[test]
    fn exactly_63_chars_is_accepted() {
        assert!(validate_oauth2_name(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn leading_trailing_whitespace_trimmed_and_accepted() {
        assert!(validate_oauth2_name("  grafana  ").is_ok());
    }
}

#[cfg(test)]
mod displayname_tests {
    use super::validate_oauth2_displayname;

    #[test]
    fn empty_is_rejected() {
        assert!(validate_oauth2_displayname("").is_err());
    }

    #[test]
    fn whitespace_only_is_rejected() {
        assert!(validate_oauth2_displayname("   ").is_err());
    }

    #[test]
    fn non_empty_is_accepted() {
        assert!(validate_oauth2_displayname("Grafana").is_ok());
    }

    #[test]
    fn single_char_is_accepted() {
        assert!(validate_oauth2_displayname("G").is_ok());
    }

    #[test]
    fn utf8_is_accepted() {
        assert!(validate_oauth2_displayname("Ångström App").is_ok());
    }

    #[test]
    fn leading_trailing_whitespace_trimmed_and_accepted() {
        assert!(validate_oauth2_displayname("  Grafana  ").is_ok());
    }
}

#[cfg(test)]
mod url_tests {
    use super::validate_landing_url;

    #[test]
    fn empty_is_rejected() {
        assert!(validate_landing_url("").is_err());
    }

    #[test]
    fn whitespace_only_is_rejected() {
        assert!(validate_landing_url("   ").is_err());
    }

    #[test]
    fn plain_word_is_rejected() {
        assert!(validate_landing_url("notaurl").is_err());
    }

    #[test]
    fn https_url_is_accepted() {
        assert!(validate_landing_url("https://grafana.example.com").is_ok());
    }

    #[test]
    fn http_url_is_accepted() {
        assert!(validate_landing_url("http://app.local/callback").is_ok());
    }

    #[test]
    fn custom_scheme_is_accepted() {
        assert!(validate_landing_url("oc://android.opencloud.eu").is_ok());
    }

    #[test]
    fn url_with_path_is_accepted() {
        assert!(validate_landing_url("https://grafana.example.com/oauth2/callback").is_ok());
    }

    #[test]
    fn url_with_port_is_accepted() {
        assert!(validate_landing_url("https://localhost:3000").is_ok());
    }
}

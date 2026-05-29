/// A parsed scope map entry from `oauth2_rs_scope_map` or `oauth2_rs_sup_scope_map`.
///
/// The raw kanidm value format is:
/// ```text
/// group-spn@domain: {"scope1", "scope2", "scope3"}
/// ```
pub struct ParsedScopeMap {
    pub group_spn: String,
    pub scopes: Vec<String>,
}

/// Parse a single scope map value string.
///
/// Returns `None` if the string does not match the expected format.
///
/// # Format
/// The expected format is:
/// ```text
/// group-spn@domain: {"scope1", "scope2"}
/// ```
///
/// Steps:
/// 1. Find the FIRST `:` and split on it. Left (trimmed) = `group_spn`.
/// 2. Right (trimmed) must be enclosed in `{...}`. Strip braces.
/// 3. Replace `{...}` with `[...]` and deserialize as `Vec<String>`.
pub fn parse_scope_map(value: &str) -> Option<ParsedScopeMap> {
    // 1. Split on the first `:`.
    let colon_pos = value.find(':')?;
    let group_spn = value[..colon_pos].trim().to_string();
    let rest = value[colon_pos + 1..].trim();

    // 2. The right side must look like `{...}`.
    let inner = rest.strip_prefix('{')?.strip_suffix('}')?;

    // 3. Replace { } with [ ] and parse as JSON array.
    let json_array = format!("[{}]", inner);
    let scopes: Vec<String> = serde_json::from_str(&json_array).ok()?;

    Some(ParsedScopeMap { group_spn, scopes })
}

#[cfg(test)]
mod tests {
    use super::parse_scope_map;

    // ── Live examples from homelab ────────────────────────────────────────────

    #[test]
    fn live_example_oauth2_proxy_users() {
        let raw = r#"oauth2-proxy-users@idm.home.amsh.dev: {"email", "groups", "openid"}"#;
        let parsed = parse_scope_map(raw).expect("should parse");
        assert_eq!(parsed.group_spn, "oauth2-proxy-users@idm.home.amsh.dev");
        let mut scopes = parsed.scopes.clone();
        scopes.sort();
        assert_eq!(scopes, ["email", "groups", "openid"]);
    }

    #[test]
    fn live_example_blinko_users() {
        let raw = r#"blinko-users@idm.home.amsh.dev: {"email", "groups", "openid", "profile"}"#;
        let parsed = parse_scope_map(raw).expect("should parse");
        assert_eq!(parsed.group_spn, "blinko-users@idm.home.amsh.dev");
        let mut scopes = parsed.scopes.clone();
        scopes.sort();
        assert_eq!(scopes, ["email", "groups", "openid", "profile"]);
    }

    #[test]
    fn live_example_minio_users() {
        let raw = r#"minio-users@idm.home.amsh.dev: {"email", "groups", "openid", "profile"}"#;
        let parsed = parse_scope_map(raw).expect("should parse");
        assert_eq!(parsed.group_spn, "minio-users@idm.home.amsh.dev");
        let mut scopes = parsed.scopes.clone();
        scopes.sort();
        assert_eq!(scopes, ["email", "groups", "openid", "profile"]);
    }

    #[test]
    fn live_example_kavita_users() {
        let raw = r#"kavita-users@idm.home.amsh.dev: {"email", "offline_access", "openid", "profile", "roles"}"#;
        let parsed = parse_scope_map(raw).expect("should parse");
        assert_eq!(parsed.group_spn, "kavita-users@idm.home.amsh.dev");
        let mut scopes = parsed.scopes.clone();
        scopes.sort();
        assert_eq!(scopes, ["email", "offline_access", "openid", "profile", "roles"]);
    }

    #[test]
    fn live_example_vault_users() {
        let raw = r#"vault-users@idm.home.amsh.dev: {"email", "groups_name", "openid", "profile"}"#;
        let parsed = parse_scope_map(raw).expect("should parse");
        assert_eq!(parsed.group_spn, "vault-users@idm.home.amsh.dev");
        let mut scopes = parsed.scopes.clone();
        scopes.sort();
        assert_eq!(scopes, ["email", "groups_name", "openid", "profile"]);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn empty_scope_set() {
        let raw = "groupx@d: {}";
        let parsed = parse_scope_map(raw).expect("should parse empty set");
        assert_eq!(parsed.group_spn, "groupx@d");
        assert!(parsed.scopes.is_empty());
    }

    #[test]
    fn single_scope() {
        let raw = r#"groupx@d: {"openid"}"#;
        let parsed = parse_scope_map(raw).expect("should parse single scope");
        assert_eq!(parsed.group_spn, "groupx@d");
        assert_eq!(parsed.scopes, ["openid"]);
    }

    #[test]
    fn malformed_no_colon() {
        let raw = "garbage no colon";
        assert!(parse_scope_map(raw).is_none(), "should fail without colon");
    }

    #[test]
    fn malformed_not_braces() {
        let raw = "group: notbraces";
        assert!(parse_scope_map(raw).is_none(), "should fail when body is not in braces");
    }

    #[test]
    fn whitespace_tolerant() {
        let raw = r#"group : { "a" , "b" }"#;
        let parsed = parse_scope_map(raw).expect("should be whitespace-tolerant");
        assert_eq!(parsed.group_spn, "group");
        let mut scopes = parsed.scopes.clone();
        scopes.sort();
        assert_eq!(scopes, ["a", "b"]);
    }
}

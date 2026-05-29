use kanidm_proto::internal::Oauth2ClaimMapJoin;

/// A parsed claim map entry from `oauth2_rs_claim_map`.
///
/// The raw kanidm wire format is:
/// ```text
/// {claim_name}:{group_spn}:{join_char}:{json_value_or_array}
/// ```
///
/// The `{join_char}` is the literal separator kanidm uses for the strategy.
/// Per `OauthClaimMapJoin::to_str()` in kanidm's `server/lib/src/value.rs`:
/// - `,` → `Oauth2ClaimMapJoin::Csv`
/// - ` ` (space) → `Oauth2ClaimMapJoin::Ssv`
/// - `;` → `Oauth2ClaimMapJoin::Array`
///
/// The values portion is JSON-encoded — either a single quoted string `"val"`
/// or a JSON array `["val1","val2"]`.
pub struct ParsedClaimMap {
    pub claim_name: String,
    pub group_spn: String,
    pub join: Oauth2ClaimMapJoin,
    pub values: Vec<String>,
}

/// Parse a single claim map value string from `oauth2_rs_claim_map`.
///
/// # Format
/// ```text
/// {claim_name}:{group_spn}:{join_char}:{json_value}
/// ```
///
/// The group SPN always contains `@` (e.g. `group@domain`). We use the `@`
/// to locate the SPN boundary rather than blindly splitting on `:`, which
/// would break because the separator character appears multiple times.
///
/// # Returns
/// `None` if the string does not match the expected format.
pub fn parse_claim_map(value: &str) -> Option<ParsedClaimMap> {
    // 1. Split off the claim name at the first ':'.
    let (claim, rest) = value.split_once(':')?;
    if claim.is_empty() {
        return None;
    }

    // 2. The group SPN extends from here until ':' after the '@'.
    //    SPNs look like `name@domain` so we find '@' then the next ':' after it.
    let at_pos = rest.find('@')?;
    let after_at = &rest[at_pos + 1..];
    let colon_after_at = after_at.find(':')?;
    let group_end = at_pos + 1 + colon_after_at;
    let group_spn = &rest[..group_end];
    if group_spn.is_empty() {
        return None;
    }

    // 3. After the group, we have ":{join_char}:{values}".
    let tail = &rest[group_end + 1..];
    let mut chars = tail.chars();
    let join_char = chars.next()?;
    // Consume the ':' separator between join_char and values.
    if chars.next() != Some(':') {
        return None;
    }
    let values_str = &tail[join_char.len_utf8() + 1..]; // join_char + ':'

    // 4. Map join_char to Oauth2ClaimMapJoin per kanidm's wire format.
    //    See `OauthClaimMapJoin::to_str()` in kanidm `server/lib/src/value.rs`.
    let join = match join_char {
        ',' => Oauth2ClaimMapJoin::Csv,
        ' ' => Oauth2ClaimMapJoin::Ssv,
        ';' => Oauth2ClaimMapJoin::Array,
        _ => return None,
    };

    // 5. Parse the values. The value portion is JSON-encoded.
    //    It may be: `"single"` or `["val1","val2"]`.
    let values: Vec<String> = if values_str.starts_with('[') {
        // Already a JSON array.
        serde_json::from_str(values_str).ok()?
    } else {
        // One or more JSON-quoted strings separated by the join char.
        // Most commonly a single `"value"`.
        values_str
            .split(join_char)
            .filter_map(|s| serde_json::from_str::<String>(s.trim()).ok())
            .collect()
    };

    Some(ParsedClaimMap {
        claim_name: claim.to_string(),
        group_spn: group_spn.to_string(),
        join,
        values,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_claim_map;
    use kanidm_proto::internal::Oauth2ClaimMapJoin;

    // ── Live homelab fixtures ─────────────────────────────────────────────────

    #[test]
    fn live_groups_opencloud_users_array() {
        // Live homelab fixture: ';' is kanidm's wire char for Array join.
        let raw = r#"groups:opencloud-users@idm.home.amsh.dev:;:"opencloud-users""#;
        let parsed = parse_claim_map(raw).expect("should parse");
        assert_eq!(parsed.claim_name, "groups");
        assert_eq!(parsed.group_spn, "opencloud-users@idm.home.amsh.dev");
        assert!(matches!(parsed.join, Oauth2ClaimMapJoin::Array));
        assert_eq!(parsed.values, vec!["opencloud-users"]);
    }

    #[test]
    fn live_oc_role_opencloud_users_ssv() {
        // Live homelab fixture: ' ' (space) is kanidm's wire char for Ssv join.
        let raw = r#"oc_role:opencloud-users@idm.home.amsh.dev: :"user""#;
        let parsed = parse_claim_map(raw).expect("should parse");
        assert_eq!(parsed.claim_name, "oc_role");
        assert_eq!(parsed.group_spn, "opencloud-users@idm.home.amsh.dev");
        assert!(matches!(parsed.join, Oauth2ClaimMapJoin::Ssv));
        assert_eq!(parsed.values, vec!["user"]);
    }

    #[test]
    fn live_policy_minio_admins_csv() {
        let raw = r#"policy:minio-admins@idm.home.amsh.dev:,:"allAccess""#;
        let parsed = parse_claim_map(raw).expect("should parse");
        assert_eq!(parsed.claim_name, "policy");
        assert_eq!(parsed.group_spn, "minio-admins@idm.home.amsh.dev");
        assert!(matches!(parsed.join, Oauth2ClaimMapJoin::Csv));
        assert_eq!(parsed.values, vec!["allAccess"]);
    }

    #[test]
    fn live_audiobooks_ssv_gg() {
        // Live homelab fixture (the one that surfaced the bug).
        let raw = r#"oc_role:audiobooks-users@idm.home.amsh.dev: :"gg""#;
        let parsed = parse_claim_map(raw).expect("should parse");
        assert_eq!(parsed.claim_name, "oc_role");
        assert_eq!(parsed.group_spn, "audiobooks-users@idm.home.amsh.dev");
        assert!(matches!(parsed.join, Oauth2ClaimMapJoin::Ssv));
        assert_eq!(parsed.values, vec!["gg"]);
    }

    // ── Synthetic / edge cases ────────────────────────────────────────────────

    #[test]
    fn array_values_json_array() {
        // Synthetic: array join strategy emits a JSON array.
        let raw = r#"role:admins@d:;:["admin","user"]"#;
        let parsed = parse_claim_map(raw).expect("should parse array values");
        assert_eq!(parsed.claim_name, "role");
        assert_eq!(parsed.group_spn, "admins@d");
        assert_eq!(parsed.values, vec!["admin", "user"]);
    }

    #[test]
    fn multiple_csv_values() {
        // Multiple values separated by the join char (csv, comma).
        let raw = r#"roles:admins@example.com:,:"admin","editor""#;
        let parsed = parse_claim_map(raw).expect("should parse multiple csv values");
        assert_eq!(parsed.claim_name, "roles");
        assert_eq!(parsed.values, vec!["admin", "editor"]);
    }

    #[test]
    fn malformed_no_colons() {
        let raw = "no_colons_at_all";
        assert!(parse_claim_map(raw).is_none(), "should return None");
    }

    #[test]
    fn malformed_empty_string() {
        let raw = "";
        assert!(parse_claim_map(raw).is_none(), "should return None for empty");
    }

    #[test]
    fn malformed_missing_at_in_group() {
        // No '@' in the group SPN → cannot find SPN boundary.
        let raw = r#"claim:nogroup:,:"value""#;
        assert!(parse_claim_map(raw).is_none(), "should return None when no @ in group");
    }

    #[test]
    fn malformed_no_json_value() {
        // join_char present but no ':' after it.
        let raw = "claim:grp@d:,";
        assert!(parse_claim_map(raw).is_none());
    }

    #[test]
    fn empty_claim_name() {
        let raw = r#":group@domain:,:"value""#;
        assert!(parse_claim_map(raw).is_none(), "empty claim name should fail");
    }

    #[test]
    fn unknown_join_char_is_rejected() {
        // 'x' is not a known join char — parser must return None rather than
        // silently falling back to Array, which could mask wire-format drift.
        assert!(parse_claim_map(r#"claim:grp@d:x:"val""#).is_none());
    }
}

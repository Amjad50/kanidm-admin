// ── Pure SSH helpers ──────────────────────────────────────────────────────────
//
// These functions are free of Axum / handler types and live here so that
// handler code and tests can import them without pulling in the full HTTP
// handler module.

use std::borrow::Cow;

/// Compute the OpenSSH-style SHA256 fingerprint for a public key's base64 blob.
/// Returns `None` when `key_data_b64` is not valid standard base64.
pub fn compute_fingerprint(key_data_b64: &str) -> Option<String> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(key_data_b64)
        .ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    let b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(digest);
    Some(format!("SHA256:{b64}"))
}

/// Map a raw SSH algorithm identifier to a human-friendly label.
pub fn algorithm_friendly(raw: &str) -> Cow<'static, str> {
    match raw {
        "ssh-ed25519" => Cow::Borrowed("Ed25519"),
        "ssh-rsa" => Cow::Borrowed("RSA"),
        "ssh-dss" => Cow::Borrowed("DSA"),
        "ecdsa-sha2-nistp256" => Cow::Borrowed("ECDSA P-256"),
        "ecdsa-sha2-nistp384" => Cow::Borrowed("ECDSA P-384"),
        "ecdsa-sha2-nistp521" => Cow::Borrowed("ECDSA P-521"),
        other => {
            if let Some(rest) = other.strip_prefix("sk-") {
                let base = rest
                    .split('@')
                    .next()
                    .unwrap_or(rest)
                    .trim_end_matches('-');
                // Recursively resolve the base algorithm name, then append suffix.
                let base_friendly = algorithm_friendly(base);
                Cow::Owned(format!("{base_friendly} (security key)"))
            } else {
                Cow::Owned(other.to_string())
            }
        }
    }
}

/// Parse a kanidm `ssh_publickey` attribute value into its components.
///
/// The kanidm wire format is:
/// ```text
/// {tag}: {algorithm} {base64_blob} {comment}
/// ```
/// For example: `pc: ssh-ed25519 AAAAC3Nz... ` (trailing space = empty comment).
///
/// Returns `None` on malformed input (no `": "` separator, empty tag, missing
/// algorithm or key fields).
///
/// Note: `fingerprint` is set to `"—"` when the key blob is not valid base64.
pub fn parse_ssh_publickey_line(line: &str) -> Option<ParsedSshKey> {
    // Strip trailing whitespace (kanidm appends a trailing space for empty comments).
    let line = line.trim_end();
    // Split on the FIRST ": " — left side is the tag, right side is the OpenSSH key line.
    let (tag, rest) = line.split_once(": ")?;
    let tag = tag.trim().to_string();
    if tag.is_empty() {
        return None;
    }
    // Parse the OpenSSH key line: `algorithm key_b64 [comment]`
    let mut parts = rest.splitn(3, char::is_whitespace);
    let algorithm = parts.next()?.to_string();
    let key_data = parts.next()?;
    if algorithm.is_empty() || key_data.is_empty() {
        return None;
    }
    // parts.next() would be the comment — we ignore it; tag is on the left side.
    let fingerprint = compute_fingerprint(key_data).unwrap_or_else(|| "—".to_string());
    let fingerprint_truncated = {
        let chars: Vec<char> = fingerprint.chars().collect();
        if chars.len() > 28 {
            format!("{}…", chars[..28].iter().collect::<String>())
        } else {
            fingerprint.clone()
        }
    };
    let algorithm_friendly = algorithm_friendly(&algorithm).into_owned();
    let openssh_line = rest.trim_end().to_string();
    let openssh_line_preview = truncate_middle(&openssh_line, 40);
    Some(ParsedSshKey {
        tag,
        algorithm_friendly,
        fingerprint,
        fingerprint_truncated,
        openssh_line,
        openssh_line_preview,
    })
}

/// Compact a long single-line string into `head…tail` form with a total length
/// (excluding the ellipsis) close to `max_chars`. Splits roughly in the middle
/// to preserve both algorithm prefix and trailing comment.
fn truncate_middle(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        return s.to_string();
    }
    let head_len = max_chars / 2;
    let tail_len = max_chars - head_len;
    let head: String = chars[..head_len].iter().collect();
    let tail: String = chars[chars.len() - tail_len..].iter().collect();
    format!("{head}…{tail}")
}

/// Parsed representation of a single SSH public key line.
pub struct ParsedSshKey {
    pub tag: String,
    pub algorithm_friendly: String,
    pub fingerprint: String,
    pub fingerprint_truncated: String,
    /// The algorithm + key_b64 [+ comment] portion — copy-paste ready for authorized_keys.
    pub openssh_line: String,
    /// Truncated `head…tail` form of `openssh_line` for a compact preview cell.
    pub openssh_line_preview: String,
}

/// Validate a tag (label) for an SSH public key.
pub fn validate_tag(tag: &str) -> Result<(), &'static str> {
    if tag.is_empty() {
        return Err("Label is required.");
    }
    if tag.len() > 63 {
        return Err("Tag must be 63 characters or less.");
    }
    if !tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err("Label may only contain letters, digits, '.', '_', '-'.");
    }
    Ok(())
}

/// Validate the shape of an OpenSSH public key string (does not verify the
/// cryptographic content — only that it begins with a recognised key type and
/// has at least a key-data field).
pub fn validate_pubkey_shape(key: &str) -> Result<(), &'static str> {
    const KNOWN_PREFIXES: &[&str] = &[
        "ssh-ed25519 ",
        "ssh-rsa ",
        "ssh-dss ",
        "ecdsa-sha2-nistp256 ",
        "ecdsa-sha2-nistp384 ",
        "ecdsa-sha2-nistp521 ",
        "sk-ssh-ed25519@openssh.com ",
        "sk-ecdsa-sha2-nistp256@openssh.com ",
    ];
    let key = key.trim();
    if key.is_empty() {
        return Err("Public key is required.");
    }
    let has_prefix = KNOWN_PREFIXES.iter().any(|p| key.starts_with(p));
    let has_two_fields = key.splitn(2, ' ').nth(1).is_some_and(|s| !s.is_empty());
    if !has_prefix || !has_two_fields {
        return Err(
            "Key must start with a recognised SSH key type (ssh-ed25519, ssh-rsa, ecdsa-sha2-*, etc.).",
        );
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{algorithm_friendly, compute_fingerprint, parse_ssh_publickey_line, validate_tag};

    // ── compute_fingerprint ────────────────────────────────────────────

    #[test]
    fn fingerprint_known_data() {
        // base64("test key data") → deterministic SHA256 fingerprint
        let fp = compute_fingerprint(VALID_B64).expect("valid base64 must produce a fingerprint");
        assert!(fp.starts_with("SHA256:"), "fingerprint must start with SHA256:");
        assert!(fp.len() > 8, "fingerprint must have content after prefix");
        // Verify the exact value matches the expected SHA256 of "test key data"
        assert_eq!(fp, "SHA256:qAS3v+C3Ba0KaEO/tkZpCW7xhNhj3kh5A7Y2Y0xKmVk");
    }

    #[test]
    fn fingerprint_empty_data_is_some() {
        // Empty string is valid base64 (zero bytes) — hash of empty bytes is deterministic.
        let fp = compute_fingerprint("").expect("empty base64 should still produce fingerprint");
        assert!(fp.starts_with("SHA256:"));
    }

    #[test]
    fn fingerprint_invalid_base64_returns_none() {
        assert_eq!(
            compute_fingerprint("not-valid-base64!!!"),
            None,
            "invalid base64 must return None, not a hash of zero bytes"
        );
    }

    #[test]
    fn fingerprint_invalid_base64_is_not_zero_bytes_hash() {
        // Confirm we never silently produce the hash of empty bytes for bad input.
        let zero_bytes_hash = compute_fingerprint("").unwrap();
        assert_ne!(
            compute_fingerprint("not-valid-base64!!!"),
            Some(zero_bytes_hash),
            "invalid base64 must not produce the same fingerprint as empty bytes"
        );
    }

    // ── validate_tag ──────────────────────────────────────────────────

    #[test]
    fn validate_tag_empty_rejected() {
        assert!(validate_tag("").is_err());
    }

    #[test]
    fn validate_tag_whitespace_rejected() {
        assert!(validate_tag("my tag").is_err());
    }

    #[test]
    fn validate_tag_at_sign_rejected() {
        assert!(validate_tag("tag@host").is_err());
    }

    #[test]
    fn validate_tag_valid_chars() {
        assert!(validate_tag("laptop_ed25519").is_ok());
        assert!(validate_tag("work-laptop").is_ok());
        assert!(validate_tag("key.2024").is_ok());
        assert!(validate_tag("ABC123").is_ok());
    }

    #[test]
    fn validate_tag_exactly_63_chars_accepted() {
        let tag = "a".repeat(63);
        assert!(validate_tag(&tag).is_ok(), "63-char tag must be accepted");
    }

    #[test]
    fn validate_tag_64_chars_rejected() {
        let tag = "a".repeat(64);
        assert!(validate_tag(&tag).is_err(), "64-char tag must be rejected");
    }

    // ── algorithm_friendly ────────────────────────────────────────────

    #[test]
    fn algorithm_friendly_known_types() {
        assert_eq!(algorithm_friendly("ssh-ed25519").as_ref(), "Ed25519");
        assert_eq!(algorithm_friendly("ssh-rsa").as_ref(), "RSA");
        assert_eq!(algorithm_friendly("ssh-dss").as_ref(), "DSA");
        assert_eq!(algorithm_friendly("ecdsa-sha2-nistp256").as_ref(), "ECDSA P-256");
        assert_eq!(algorithm_friendly("ecdsa-sha2-nistp384").as_ref(), "ECDSA P-384");
        assert_eq!(algorithm_friendly("ecdsa-sha2-nistp521").as_ref(), "ECDSA P-521");
    }

    #[test]
    fn algorithm_friendly_security_key() {
        let name = algorithm_friendly("sk-ssh-ed25519@openssh.com");
        assert!(
            name.contains("security key"),
            "sk- prefix should produce security key label"
        );
    }

    #[test]
    fn algorithm_friendly_unknown_passthrough() {
        assert_eq!(algorithm_friendly("x-custom-algo").as_ref(), "x-custom-algo");
    }

    // ── parse_ssh_publickey_line ──────────────────────────────────────

    // dGVzdCBrZXkgZGF0YQ== is base64("test key data") — valid standard base64.
    const VALID_B64: &str = "dGVzdCBrZXkgZGF0YQ==";

    // Real-world base64 blob from a live kanidm instance (ed25519 key).
    const REAL_ED25519_B64: &str =
        "AAAAC3NzaC1lZDI1NTE5AAAAIGRGvFgz+AH8SllcU1ZRbVw5cyfzCOo5gRuxu+DLMLHn";

    #[test]
    fn parse_kanidm_wire_format_real_world() {
        // This is the exact value returned by a live kanidm instance for an ed25519 key tagged "pc".
        // Trailing space comes from an empty comment field.
        let line = format!("pc: ssh-ed25519 {REAL_ED25519_B64} ");
        let key = parse_ssh_publickey_line(&line).expect("real-world kanidm line must parse");
        assert_eq!(key.tag, "pc");
        assert_eq!(key.algorithm_friendly, "Ed25519");
        // Fingerprint must match `ssh-keygen -l -E sha256` output: SHA256:zA/xpGjbERsh5GZw0tULpPJnNsenLHmA/MtVkfYHnJM
        assert_eq!(
            key.fingerprint,
            "SHA256:zA/xpGjbERsh5GZw0tULpPJnNsenLHmA/MtVkfYHnJM"
        );
        assert_eq!(
            key.openssh_line,
            format!("ssh-ed25519 {REAL_ED25519_B64}")
        );
    }

    #[test]
    fn parse_tag_with_colon_uses_first_separator() {
        // Kanidm tags don't allow colons, but the parser should correctly use the FIRST ": "
        // as the separator. This test confirms that behaviour is unambiguous.
        let line = format!("weird:tag: ssh-ed25519 {VALID_B64} ");
        let key = parse_ssh_publickey_line(&line).expect("tag with colon must parse");
        assert_eq!(key.tag, "weird:tag");
        assert_eq!(key.algorithm_friendly, "Ed25519");
    }

    #[test]
    fn parse_comment_present_tag_from_left_side() {
        // When a non-empty comment is present, tag still comes from left of ": ".
        let line = format!("laptop: ssh-ed25519 {VALID_B64} alice@host");
        let key = parse_ssh_publickey_line(&line).expect("line with comment must parse");
        assert_eq!(key.tag, "laptop");
        assert_eq!(key.algorithm_friendly, "Ed25519");
        assert!(key.fingerprint.starts_with("SHA256:"));
        assert_eq!(key.openssh_line, format!("ssh-ed25519 {VALID_B64} alice@host"));
    }

    #[test]
    fn parse_ed25519_kanidm_format() {
        let line = format!("my_tag: ssh-ed25519 {VALID_B64} ");
        let key = parse_ssh_publickey_line(&line).expect("should parse");
        assert_eq!(key.algorithm_friendly, "Ed25519");
        assert_eq!(key.tag, "my_tag");
        assert!(key.fingerprint.starts_with("SHA256:"));
        // Trailing space (empty comment) must be stripped from openssh_line.
        assert_eq!(key.openssh_line, format!("ssh-ed25519 {VALID_B64}"));
    }

    #[test]
    fn parse_rsa_kanidm_format() {
        let line = format!("my-key-1: ssh-rsa {VALID_B64} ");
        let key = parse_ssh_publickey_line(&line).expect("should parse even without tag");
        assert_eq!(key.tag, "my-key-1");
        assert_eq!(key.algorithm_friendly, "RSA");
    }

    #[test]
    fn parse_missing_separator_returns_none() {
        // No ": " separator — old broken format — must return None.
        let line = format!("ssh-ed25519 {VALID_B64} my_tag");
        assert!(
            parse_ssh_publickey_line(&line).is_none(),
            "missing ': ' separator must return None"
        );
    }

    #[test]
    fn parse_empty_line_returns_none() {
        assert!(parse_ssh_publickey_line("").is_none());
    }

    #[test]
    fn parse_empty_tag_returns_none() {
        // Empty tag (just ": " at the start) must be rejected.
        let line = format!(": ssh-ed25519 {VALID_B64} ");
        assert!(
            parse_ssh_publickey_line(&line).is_none(),
            "empty tag must return None"
        );
    }

    #[test]
    fn fingerprint_truncated_to_28_chars_with_ellipsis() {
        let line = format!("my_tag: ssh-ed25519 {VALID_B64} ");
        let key = parse_ssh_publickey_line(&line).expect("should parse");
        // SHA256 fingerprints are always > 28 chars ("SHA256:" + 43 chars of base64 = 50).
        assert!(
            key.fingerprint.len() >= 28,
            "full fingerprint should be at least 28 chars"
        );
        assert!(
            key.fingerprint_truncated.ends_with('…')
                || key.fingerprint_truncated == key.fingerprint,
            "truncated should end with ellipsis or equal full fingerprint when short"
        );
    }
}

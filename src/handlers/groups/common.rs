use kanidm_client::{ClientError, StatusCode};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::attr_first;
use crate::AppState;

// ── Validation ────────────────────────────────────────────────────────────────

/// Group names follow similar rules to person names but allow underscore as
/// the first character to support service-style groups like `_svc_build`.
pub fn validate_group_name(s: &str) -> Result<(), &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Group name is required.");
    }
    if s.len() > 63 {
        return Err("Group name must be 63 characters or less.");
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_lowercase() && first != '_' {
        return Err("Group name must start with a lowercase letter or underscore.");
    }
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-')) {
        return Err("Group name may only contain lowercase letters, digits, '.', '_', '-'.");
    }
    Ok(())
}

// ── Shared API helpers ────────────────────────────────────────────────────────

pub(super) async fn fetch_group(
    state: &AppState,
    user: &AdminUser,
    id: &str,
) -> AppResult<kanidm_proto::v1::Entry> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    client
        .idm_group_get(id)
        .await
        .map_err(|e| AppError::Kanidm(format!("group get failed: {e:?}")))?
        .ok_or(AppError::NotFound)
}

pub(super) fn friendly_error(context: &str, e: &ClientError) -> String {
    match e {
        ClientError::Http(StatusCode::CONFLICT, _, _) => {
            format!("Could not {context}: this name is already in use.")
        }
        ClientError::Http(StatusCode::NOT_FOUND, _, _) => {
            format!("Could not {context}: resource not found.")
        }
        _ => format!("Could not {context}: {e:?}"),
    }
}

// ── Computed group header ─────────────────────────────────────────────────────

/// Shared header data rendered above all group detail tabs.
pub struct GroupHeader {
    pub name: String,
    pub spn: String,
    pub spn_or_uuid: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub has_policy: bool,
    pub is_builtin: bool,
    pub is_dynamic: bool,
}

pub(super) fn compute_header(entry: &kanidm_proto::v1::Entry) -> GroupHeader {
    use crate::kanidm::entry::{attr_all, spn_or_uuid};

    let name = attr_first(entry, "name").unwrap_or_default();
    let spn = attr_first(entry, "spn").unwrap_or_default();
    let description = attr_first(entry, "description");

    let classes = attr_all(entry, "class");
    let has_policy = classes.iter().any(|c| c == "account_policy");
    let is_builtin = classes.iter().any(|c| c == "builtin");
    let is_dynamic = classes.iter().any(|c| c == "dyngroup");

    // Static member count — prefer `member` for normal groups, `dynmember` for dyngroups
    let member_count = if is_dynamic {
        attr_all(entry, "dynmember").len()
    } else {
        attr_all(entry, "member").len()
    };

    GroupHeader {
        name,
        spn,
        spn_or_uuid: spn_or_uuid(entry),
        description,
        member_count,
        has_policy,
        is_builtin,
        is_dynamic,
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Derive 1–2 uppercase initials from a Kanidm SPN (`user@domain` or plain name).
pub(super) fn spn_initials(spn: &str) -> String {
    let name_part = spn.split('@').next().unwrap_or(spn);
    let parts: Vec<&str> = name_part
        .split(['.', '_', '-'])
        .filter(|p| !p.is_empty())
        .collect();
    match parts.len() {
        0 => "?".to_string(),
        1 => parts[0]
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default(),
        _ => {
            let first = parts[0].chars().next().unwrap_or('?');
            let last = parts[parts.len() - 1].chars().next().unwrap_or('?');
            format!("{}{}", first.to_uppercase(), last.to_uppercase())
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::validate_group_name;

    #[test]
    fn empty_is_rejected() {
        assert!(validate_group_name("").is_err());
    }

    #[test]
    fn whitespace_only_is_rejected() {
        assert!(validate_group_name("   ").is_err());
    }

    #[test]
    fn starts_with_uppercase_is_rejected() {
        assert!(validate_group_name("Admins").is_err());
    }

    #[test]
    fn starts_with_digit_is_rejected() {
        assert!(validate_group_name("1group").is_err());
    }

    #[test]
    fn simple_lowercase_accepted() {
        assert!(validate_group_name("developers").is_ok());
    }

    #[test]
    fn underscore_start_accepted() {
        assert!(validate_group_name("_svc_build").is_ok());
    }

    #[test]
    fn with_hyphen_and_dot_accepted() {
        assert!(validate_group_name("dev-ops.team").is_ok());
    }

    #[test]
    fn uppercase_in_middle_rejected() {
        assert!(validate_group_name("devOps").is_err());
    }

    #[test]
    fn over_63_chars_rejected() {
        assert!(validate_group_name(&"a".repeat(64)).is_err());
    }

    #[test]
    fn exactly_63_chars_accepted() {
        assert!(validate_group_name(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn at_sign_rejected() {
        assert!(validate_group_name("dev@ops").is_err());
    }

    #[test]
    fn space_rejected() {
        assert!(validate_group_name("dev ops").is_err());
    }
}

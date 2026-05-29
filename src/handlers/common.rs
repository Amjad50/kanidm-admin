use kanidm_client::{ClientError, StatusCode};
use kanidm_proto::attribute::Attribute;
use kanidm_proto::internal::{OperationError, PluginError};

// ── Shared handler-tier types ─────────────────────────────────────────────────

/// A single row in a multi-email input list.
pub struct EmailRow {
    pub value: String,
    pub is_primary: bool,
}

pub fn emails_to_rows(emails: &[String]) -> Vec<EmailRow> {
    emails
        .iter()
        .enumerate()
        .map(|(i, v)| EmailRow { value: v.clone(), is_primary: i == 0 })
        .collect()
}

// ── User-facing error messages ────────────────────────────────────────────────

/// Map a kanidm `ClientError` to a short user-friendly sentence.
///
/// `context` is a short verb phrase like `"create person"` or `"add SSH key"`,
/// used as a prefix only when kanidm gave us nothing more structured. When the
/// server returned a typed `OperationError` we use that instead, since it
/// usually tells us exactly which attribute or rule was violated.
///
/// This function emits `tracing::warn!` with the full raw error before
/// returning a message, so the underlying detail is always recoverable from
/// logs even if the user only sees the friendly summary.
pub fn friendly_client_error(context: &str, e: &ClientError) -> String {
    log_client_error(context, e);

    if let ClientError::Http(status, op_err, body) = e {
        if let Some(op) = op_err {
            if let Some(msg) = friendly_operation_error(op) {
                return msg;
            }
        }
        // No structured OperationError. Fall back on body-sniffing for the
        // common conflict case before the generic per-status message.
        if let Some(msg) = body_sniff_message(*status, body, context) {
            return msg;
        }
    }

    match e {
        ClientError::Http(StatusCode::CONFLICT, _, _) => {
            format!("Could not {context}: a value already in use by another entry.")
        }
        ClientError::Http(StatusCode::NOT_FOUND, _, _) => {
            format!("Could not {context}: resource not found.")
        }
        ClientError::Http(StatusCode::UNAUTHORIZED, _, _) => {
            format!("Could not {context}: not authenticated.")
        }
        ClientError::Http(StatusCode::FORBIDDEN, _, _) => {
            format!("Could not {context}: not authorised.")
        }
        _ => format!("Could not {context}: {e:?}"),
    }
}

/// Dump the full client error so operators can see exactly what kanidm sent
/// even when the user only sees the polished summary.
fn log_client_error(context: &str, e: &ClientError) {
    match e {
        ClientError::Http(status, op_err, body) => {
            tracing::warn!(
                context = %context,
                http_status = %status,
                operation_error = ?op_err,
                body = %body,
                "kanidm rejected the request"
            );
        }
        other => {
            tracing::warn!(
                context = %context,
                error = ?other,
                "kanidm client error (non-HTTP)"
            );
        }
    }
}

/// Translate a structured `OperationError` into a complete user-facing sentence.
/// Returns `None` when the variant has no good translation; the caller falls
/// back to its `context`-prefixed generic message.
fn friendly_operation_error(op: &OperationError) -> Option<String> {
    match op {
        OperationError::AttributeUniqueness(attrs) => {
            let names: Vec<&'static str> = attrs.iter().map(attribute_label).collect();
            Some(match names.as_slice() {
                [] => "A value you entered is already taken by another entry.".to_string(),
                [one] => format!("That {one} is already taken by another entry."),
                many => format!(
                    "These values are already taken by another entry: {}.",
                    many.join(", ")
                ),
            })
        }
        OperationError::UniqueConstraintViolation => {
            Some("A value you entered is already taken by another entry.".to_string())
        }
        OperationError::DuplicateKey | OperationError::DuplicateLabel => {
            Some("That label or key is already in use on this entry.".to_string())
        }
        OperationError::PasswordQuality(feedback) => {
            let parts: Vec<String> = feedback.iter().map(|f| format!("{f}")).collect();
            Some(format!("Password is not strong enough: {}.", parts.join("; ")))
        }
        OperationError::Plugin(PluginError::Base(msg))
        | OperationError::Plugin(PluginError::ReferentialIntegrity(msg))
        | OperationError::Plugin(PluginError::CredImport(msg)) => Some(msg.clone()),
        OperationError::SchemaViolation(_) => {
            Some("The value does not match the schema for this field.".to_string())
        }
        OperationError::NoMatchingEntries => Some("No matching entry was found.".to_string()),
        OperationError::AccessDenied | OperationError::NotAuthorised => {
            Some("You don't have permission to perform that action.".to_string())
        }
        OperationError::NotAuthenticated => Some("You are not authenticated.".to_string()),
        OperationError::SystemProtectedObject | OperationError::SystemProtectedAttribute => {
            Some("That target is system-protected and cannot be modified.".to_string())
        }
        OperationError::ValueDenyName => Some(
            "That name is on the system denylist (e.g. a reserved word). Pick another.".to_string(),
        ),
        OperationError::InvalidAttributeName(name) | OperationError::InvalidAttribute(name) => {
            Some(format!("Invalid attribute name: {name}."))
        }
        OperationError::ResourceLimit => {
            Some("The request exceeded a server resource limit.".to_string())
        }
        _ => None,
    }
}

/// When the server returned an HTTP error without a structured `OperationError`
/// (the response body is usually just an OperationID for log correlation),
/// use what we know about the request to produce a specific message.
///
/// The `context` strings are stable in this codebase — `"create person"`,
/// `"set additional fields"`, etc. — so we can match on them to disambiguate
/// what most likely went wrong.
fn body_sniff_message(status: StatusCode, body: &str, context: &str) -> Option<String> {
    let trimmed = body.trim();

    // The kanidm body for 409 with no structured OperationError is typically a
    // bare UUID (the OperationID). Some flows attach a JSON-encoded enum body
    // like "attributeuniqueness" — try that first.
    let lower = trimmed.to_ascii_lowercase();
    if status == StatusCode::CONFLICT
        && (lower.contains("attributeuniqueness")
            || lower.contains("unique")
            || lower.contains("duplicate"))
    {
        if lower.contains("\"name\"") || lower.contains(" name ") {
            return Some("That username is already taken by another entry.".to_string());
        }
        if lower.contains("\"mail\"") {
            return Some("That email is already taken by another entry.".to_string());
        }
        if lower.contains("\"spn\"") {
            return Some("That SPN is already taken by another entry.".to_string());
        }
    }

    // Context-based fallback. The body had no useful detail, but we know which
    // operation we just tried. These are educated guesses, not certainties —
    // hence the "most often" hedge in the wording.
    if status == StatusCode::CONFLICT {
        return Some(context_specific_conflict_message(context));
    }

    None
}

/// Best-effort message for a 409 when we have no other signal. The verb phrase
/// in `context` tells us which operation just failed.
fn context_specific_conflict_message(context: &str) -> String {
    match context {
        "create person" | "create group" => {
            format!("That name is already taken — pick a different one.")
        }
        "set additional fields" | "update person" => {
            "One of the email addresses you entered is already taken by another account."
                .to_string()
        }
        "set group description" => {
            "Could not set the group description (server reported a conflict).".to_string()
        }
        "set group mail" => {
            "One of the email addresses you entered is already in use.".to_string()
        }
        "update mail" => {
            "One of the email addresses you entered is already taken by another account."
                .to_string()
        }
        "rename group" | "update entry managed by" => {
            "That name is already taken by another group.".to_string()
        }
        "create oauth2 client" => {
            "That OAuth2 client name is already taken. Pick another.".to_string()
        }
        "rename oauth2 client" => "That OAuth2 client name is already taken.".to_string(),
        "update oauth2 client" => {
            "Could not update the OAuth2 client (server reported a conflict).".to_string()
        }
        "add redirect URL" => {
            "That redirect URL is already configured on this client.".to_string()
        }
        "reset oauth2 secret" => {
            "Could not reset the secret (server reported a conflict).".to_string()
        }
        "add SSH key" => "An SSH key with that label already exists on this account.".to_string(),
        "add members" => {
            "One of the members you tried to add already belongs to this group, or doesn't exist."
                .to_string()
        }
        _ => format!("Could not {context}: a value already in use by another entry."),
    }
}

/// Map kanidm attribute identifiers to short user-facing labels. Falls back
/// to a generic placeholder when the attribute is one we haven't named.
fn attribute_label(attr: &Attribute) -> &'static str {
    match attr.as_str() {
        "name" => "username",
        "displayname" => "display name",
        "spn" => "SPN",
        "uuid" => "UUID",
        "mail" => "email",
        "legalname" => "legal name",
        "description" => "description",
        "entry_managed_by" => "managed-by group",
        "gidnumber" => "GID number",
        "loginshell" => "login shell",
        "radius_secret" => "RADIUS secret",
        "ssh_publickey" => "SSH public key",
        "oauth2_rs_name" => "OAuth2 client name",
        "oauth2_rs_origin" => "OAuth2 origin",
        "oauth2_rs_origin_landing" => "OAuth2 landing URL",
        "class" => "class",
        "member" => "member",
        "memberof" => "group membership",
        "credential_type_minimum" => "credential type minimum",
        "auth_password_minimum_length" => "password minimum length",
        "authsession_expiry" => "auth session expiry",
        "privilege_expiry" => "privilege expiry",
        _ => "value",
    }
}

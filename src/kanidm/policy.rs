/// Static metadata for every account-policy field that can be set on a group.
///
/// The template iterates this slice — adding a new policy field means adding
/// one entry here.
pub struct PolicyField {
    pub key: &'static str,
    pub label: &'static str,
    pub helper: &'static str,
    pub kind: PolicyKind,
    /// Display value shown when the attr is absent from the entry (kanidm default).
    pub default: &'static str,
}

pub enum PolicyKind {
    Int,
    Seconds,
    Bool,
    /// Ordered list of valid string values; first is displayed as the current
    /// value when no attr is present.
    Enum(&'static [&'static str]),
    /// Freeform JSON blob (webauthn attestation CA list).
    JsonBlob,
}

pub const POLICY_FIELDS: &[PolicyField] = &[
    PolicyField {
        key: "credential_type_minimum",
        label: "Credential type minimum",
        helper: "Minimum credential quality required for members of this group.",
        kind: PolicyKind::Enum(&["any", "mfa", "passkey", "attested_passkey"]),
        default: "any",
    },
    PolicyField {
        key: "auth_password_minimum_length",
        label: "Password minimum length",
        helper: "Minimum number of characters for a member's password. kanidm default: 10.",
        kind: PolicyKind::Int,
        default: "10",
    },
    PolicyField {
        key: "authsession_expiry",
        label: "Auth session expiry",
        helper: "Maximum duration (seconds) of a sign-in session. kanidm default: 86400 (24 hours).",
        kind: PolicyKind::Seconds,
        default: "86400",
    },
    PolicyField {
        key: "privilege_expiry",
        label: "Privilege session expiry",
        helper: "Duration (seconds) of an elevated re-auth session. kanidm default: 600 (10 minutes).",
        kind: PolicyKind::Seconds,
        default: "600",
    },
    PolicyField {
        key: "allow_primary_cred_fallback",
        label: "Allow primary credential fallback",
        helper: "Allow members to use their primary password for POSIX / PAM authentication. kanidm default: false.",
        kind: PolicyKind::Bool,
        default: "false",
    },
    PolicyField {
        key: "limit_search_max_results",
        label: "Limit search max results",
        helper: "Maximum number of results returned for a search performed by members of this group. kanidm default: 1024.",
        kind: PolicyKind::Int,
        default: "1024",
    },
    PolicyField {
        key: "limit_search_max_filter_test",
        label: "Limit search max filter test",
        helper: "Maximum number of entries tested against a search filter. kanidm default: 2048.",
        kind: PolicyKind::Int,
        default: "2048",
    },
    PolicyField {
        key: "webauthn_attestation_ca_list",
        label: "WebAuthn attestation CA list",
        helper: "JSON CA list following FIDO metadata service format. Restricts passkey registration to certified devices.",
        kind: PolicyKind::JsonBlob,
        default: "",
    },
];

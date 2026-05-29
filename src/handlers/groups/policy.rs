use axum::Form;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_all, attr_first};
use crate::kanidm::policy::{POLICY_FIELDS, PolicyField, PolicyKind};

use super::common::{GroupHeader, compute_header, fetch_group, friendly_error};
use super::detail::{TabContent, format_seconds, render_detail};

// ── Policy data ───────────────────────────────────────────────────────────────

/// A single policy field with its current value resolved from the entry.
pub struct PolicyFieldView {
    pub key: &'static str,
    pub label: &'static str,
    pub helper: &'static str,
    pub kind_label: &'static str,
    /// The raw string value from the entry, or None if using kanidm default.
    pub current_value: Option<String>,
    /// Empty string when None — safe for direct use in form value attrs.
    pub current_value_str: String,
    /// Human-friendly display of the value (or default hint).
    pub display_value: String,
    /// Whether this field is set (non-default).
    pub is_set: bool,
    /// For enum fields: all valid options.
    pub enum_options: Vec<EnumOption>,
    /// Kind discriminant for template matching.
    pub kind: PolicyFieldKind,
    /// Default value for display when not set.
    pub default: &'static str,
}

pub struct EnumOption {
    pub value: &'static str,
    pub selected: bool,
}

#[derive(PartialEq, Eq)]
pub enum PolicyFieldKind {
    Int,
    Seconds,
    Bool,
    Enum,
    JsonBlob,
}

pub struct PolicyData {
    pub fields: Vec<PolicyFieldView>,
    pub customized_count: usize,
    pub error: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn resolve_field(entry: &kanidm_proto::v1::Entry, field: &PolicyField) -> PolicyFieldView {
    let current_value = attr_first(entry, field.key);
    let is_set = current_value.is_some();

    let display_value = match &current_value {
        None => format!("Default — {}", field.default),
        Some(v) => match &field.kind {
            PolicyKind::Seconds => {
                let secs: u64 = v.parse().unwrap_or(0);
                format!("{v} s ({})", format_seconds(secs))
            }
            PolicyKind::Int => v.clone(),
            PolicyKind::Bool => {
                if v == "true" {
                    "Enabled".to_string()
                } else {
                    "Disabled".to_string()
                }
            }
            PolicyKind::Enum(_) => v.clone(),
            PolicyKind::JsonBlob => "Configured".to_string(),
        },
    };

    let enum_options = match &field.kind {
        PolicyKind::Enum(opts) => opts
            .iter()
            .map(|opt| EnumOption {
                value: opt,
                selected: current_value.as_deref() == Some(opt),
            })
            .collect(),
        _ => Vec::new(),
    };

    let kind = match &field.kind {
        PolicyKind::Int => PolicyFieldKind::Int,
        PolicyKind::Seconds => PolicyFieldKind::Seconds,
        PolicyKind::Bool => PolicyFieldKind::Bool,
        PolicyKind::Enum(_) => PolicyFieldKind::Enum,
        PolicyKind::JsonBlob => PolicyFieldKind::JsonBlob,
    };

    let kind_label = match &field.kind {
        PolicyKind::Int => "integer",
        PolicyKind::Seconds => "seconds",
        PolicyKind::Bool => "boolean",
        PolicyKind::Enum(_) => "enum",
        PolicyKind::JsonBlob => "json",
    };

    let current_value_str = current_value.clone().unwrap_or_default();

    PolicyFieldView {
        key: field.key,
        label: field.label,
        helper: field.helper,
        kind_label,
        current_value_str,
        current_value,
        display_value,
        is_set,
        enum_options,
        kind,
        default: field.default,
    }
}

pub(super) fn build_policy_data(entry: &kanidm_proto::v1::Entry) -> PolicyData {
    let fields: Vec<PolicyFieldView> = POLICY_FIELDS
        .iter()
        .map(|f| resolve_field(entry, f))
        .collect();

    let customized_count = fields.iter().filter(|f| f.is_set).count();
    PolicyData {
        fields,
        customized_count,
        error: None,
    }
}

/// Build the policy data with an inline error banner. Re-fetches the entry to
/// reflect any partial state, then attaches the message.
async fn build_policy_data_with_error(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    error: String,
) -> AppResult<(GroupHeader, PolicyData)> {
    let entry = fetch_group(state, user, id).await?;
    let group = compute_header(&entry);
    let mut data = build_policy_data(&entry);
    data.error = Some(error);
    Ok((group, data))
}

// ── Set/reset form ────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct SetFieldForm {
    pub value: String,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /groups/{id}/policy
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let data = build_policy_data(&entry);
    let tab_content = TabContent::Policy(data);
    render_detail(is_htmx, user, group, "policy", tab_content)
}

/// POST /groups/{id}/policy/{field}
pub async fn set_field(
    State(state): State<AppState>,
    Path((id, field)): Path<(String, String)>,
    user: AdminUser,
    Form(form): Form<SetFieldForm>,
) -> AppResult<Response> {
    // Auto-enable account_policy class if not already present
    let entry = fetch_group(&state, &user, &id).await?;
    let classes = attr_all(&entry, "class");
    let has_policy = classes.iter().any(|c| c == "account_policy");

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    if !has_policy && let Err(e) = client.group_account_policy_enable(&id).await {
        tracing::warn!(group = %id, error = ?e, "failed to enable account policy");
        let msg = friendly_error("enable account policy", &e);
        let (group, data) = build_policy_data_with_error(&state, &user, &id, msg).await?;
        return render_policy_fragment(group, data);
    }

    let value = form.value.trim().to_string();

    let parse_int = |label: &str| -> Result<u32, String> {
        value
            .parse::<u32>()
            .map_err(|_| format!("{label} must be a positive integer"))
    };

    let result = match field.as_str() {
        "authsession_expiry" => match parse_int("authsession_expiry") {
            Ok(v) => {
                client
                    .group_account_policy_authsession_expiry_set(&id, v)
                    .await
            }
            Err(msg) => {
                let (g, d) = build_policy_data_with_error(&state, &user, &id, msg).await?;
                return render_policy_fragment(g, d);
            }
        },
        "credential_type_minimum" => {
            client
                .group_account_policy_credential_type_minimum_set(&id, &value)
                .await
        }
        "auth_password_minimum_length" => match parse_int("auth_password_minimum_length") {
            Ok(v) => {
                client
                    .group_account_policy_password_minimum_length_set(&id, v)
                    .await
            }
            Err(msg) => {
                let (g, d) = build_policy_data_with_error(&state, &user, &id, msg).await?;
                return render_policy_fragment(g, d);
            }
        },
        "privilege_expiry" => match parse_int("privilege_expiry") {
            Ok(v) => {
                client
                    .group_account_policy_privilege_expiry_set(&id, v)
                    .await
            }
            Err(msg) => {
                let (g, d) = build_policy_data_with_error(&state, &user, &id, msg).await?;
                return render_policy_fragment(g, d);
            }
        },
        "webauthn_attestation_ca_list" => {
            client
                .group_account_policy_webauthn_attestation_set(&id, &value)
                .await
        }
        "limit_search_max_results" => match parse_int("limit_search_max_results") {
            Ok(v) => {
                client
                    .group_account_policy_limit_search_max_results(&id, v)
                    .await
            }
            Err(msg) => {
                let (g, d) = build_policy_data_with_error(&state, &user, &id, msg).await?;
                return render_policy_fragment(g, d);
            }
        },
        "limit_search_max_filter_test" => match parse_int("limit_search_max_filter_test") {
            Ok(v) => {
                client
                    .group_account_policy_limit_search_max_filter_test(&id, v)
                    .await
            }
            Err(msg) => {
                let (g, d) = build_policy_data_with_error(&state, &user, &id, msg).await?;
                return render_policy_fragment(g, d);
            }
        },
        "allow_primary_cred_fallback" => {
            let allow = value == "true";
            client
                .group_account_policy_allow_primary_cred_fallback(&id, allow)
                .await
        }
        _ => {
            let (g, d) = build_policy_data_with_error(
                &state,
                &user,
                &id,
                "Unknown policy field".to_string(),
            )
            .await?;
            return render_policy_fragment(g, d);
        }
    };

    if let Err(e) = result {
        tracing::warn!(group = %id, field = %field, error = ?e, "failed to set policy field");
        let msg = friendly_error(&format!("set {field}"), &e);
        let (group, data) = build_policy_data_with_error(&state, &user, &id, msg).await?;
        return render_policy_fragment(group, data);
    }

    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let data = build_policy_data(&entry);

    render_policy_fragment(group, data)
}

/// POST /groups/{id}/policy/{field}/reset
pub async fn reset_field(
    State(state): State<AppState>,
    Path((id, field)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let result = match field.as_str() {
        "authsession_expiry" => {
            client
                .group_account_policy_authsession_expiry_reset(&id)
                .await
        }
        "credential_type_minimum" => {
            client
                .idm_group_purge_attr(&id, "credential_type_minimum")
                .await
        }
        "auth_password_minimum_length" => {
            client
                .group_account_policy_password_minimum_length_reset(&id)
                .await
        }
        "privilege_expiry" => {
            client
                .group_account_policy_privilege_expiry_reset(&id)
                .await
        }
        "webauthn_attestation_ca_list" => {
            client
                .group_account_policy_webauthn_attestation_reset(&id)
                .await
        }
        "limit_search_max_results" => {
            client
                .group_account_policy_limit_search_max_results_reset(&id)
                .await
        }
        "limit_search_max_filter_test" => {
            client
                .group_account_policy_limit_search_max_filter_test_reset(&id)
                .await
        }
        "allow_primary_cred_fallback" => {
            client
                .idm_group_purge_attr(&id, "allow_primary_cred_fallback")
                .await
        }
        _ => {
            let (g, d) = build_policy_data_with_error(
                &state,
                &user,
                &id,
                "Unknown policy field".to_string(),
            )
            .await?;
            return render_policy_fragment(g, d);
        }
    };

    if let Err(e) = result {
        tracing::warn!(group = %id, field = %field, error = ?e, "failed to reset policy field");
        let msg = friendly_error(&format!("reset {field}"), &e);
        let (group, data) = build_policy_data_with_error(&state, &user, &id, msg).await?;
        return render_policy_fragment(group, data);
    }

    // Re-fetch and return updated policy fragment
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let data = build_policy_data(&entry);

    render_policy_fragment(group, data)
}

fn render_policy_fragment(group: GroupHeader, data: PolicyData) -> AppResult<Response> {
    use askama::Template;

    #[derive(Template)]
    #[template(path = "groups/_tab_policy_content.html")]
    struct PolicyContentFragment<'a> {
        p: &'a PolicyData,
        group: &'a GroupHeader,
    }

    let html = askama::Template::render(&PolicyContentFragment {
        p: &data,
        group: &group,
    })
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

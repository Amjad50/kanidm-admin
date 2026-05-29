use axum::extract::{Path, State};
use axum::response::Response;
use axum_extra::extract::Form;
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::friendly_client_error;
use crate::kanidm::entry::attr_first;
use crate::AppState;

use super::detail::{compute_header, fetch_oauth2_entry, render_detail, TabContent};

// ── Advanced tab data ─────────────────────────────────────────────────────────

pub struct AdvancedData {
    pub oauth2_id: String,
    /// `None` means "server default" (attribute absent or unparseable).
    pub refresh_expiry_seconds: Option<u32>,
    pub device_flow_enabled: bool,
    pub error: Option<String>,
}

// ── Data builder ──────────────────────────────────────────────────────────────

fn build_advanced_data(
    id: &str,
    entry: &kanidm_proto::v1::Entry,
    error: Option<String>,
) -> AdvancedData {
    // oauth2_refresh_token_expiry — stored as a uint string, e.g. "2592000"
    let refresh_expiry_seconds = attr_first(entry, "oauth2_refresh_token_expiry")
        .and_then(|v| v.parse::<u32>().ok());

    // oauth2_device_flow_enable — stored as "true" / absent
    let device_flow_enabled = attr_first(entry, "oauth2_device_flow_enable")
        .map(|v| v == "true")
        .unwrap_or(false);

    AdvancedData {
        oauth2_id: id.to_string(),
        refresh_expiry_seconds,
        device_flow_enabled,
        error,
    }
}

// ── Form struct ───────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct UpdateForm {
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub value: String,
    /// Checkbox: present as "on" when checked, absent when unchecked.
    #[serde(default)]
    pub enabled: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/advanced
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_advanced_data(&id, &entry, None);
    render_detail(is_htmx, user, header, "advanced", TabContent::Advanced(data))
}

/// POST /oauth2/{id}/advanced
pub async fn update(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<UpdateForm>,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let result: Result<(), kanidm_client::ClientError> = match form.field.as_str() {
        "refresh_expiry" => {
            let trimmed = form.value.trim();
            let parsed: Option<u32> = if trimmed.is_empty() {
                // Blank → clear the attribute (use server default)
                None
            } else {
                match trimmed.parse::<u32>() {
                    Ok(n) => Some(n),
                    Err(_) => {
                        // Invalid input — re-render with error
                        let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                        let header = compute_header(&state, &entry);
                        let data = build_advanced_data(
                            &id,
                            &entry,
                            Some(
                                "Refresh token expiry must be a non-negative integer or blank to use the default."
                                    .to_string(),
                            ),
                        );
                        return render_detail(
                            is_htmx,
                            user,
                            header,
                            "advanced",
                            TabContent::Advanced(data),
                        );
                    }
                }
            };
            client
                .idm_oauth2_rs_set_refresh_token_expiry(&id, parsed)
                .await
        }

        "device_flow" => {
            let enabled = form.enabled.as_deref() == Some("on");
            client
                .idm_oauth2_client_device_flow_update(&id, enabled)
                .await
        }

        other => {
            tracing::warn!(field = %other, "unknown advanced update field");
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_advanced_data(
                &id,
                &entry,
                Some(format!("Unknown field: {other}")),
            );
            return render_detail(
                is_htmx,
                user,
                header,
                "advanced",
                TabContent::Advanced(data),
            );
        }
    };

    match result {
        Ok(()) => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_advanced_data(&id, &entry, None);
            render_detail(is_htmx, user, header, "advanced", TabContent::Advanced(data))
        }
        Err(e) => {
            let msg = friendly_client_error("update advanced settings", &e);
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_advanced_data(&id, &entry, Some(msg));
            render_detail(is_htmx, user, header, "advanced", TabContent::Advanced(data))
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use kanidm_proto::v1::Entry;

    use super::build_advanced_data;

    fn make_entry(attrs: &[(&str, &[&str])]) -> Entry {
        let mut map = BTreeMap::new();
        for (key, vals) in attrs {
            map.insert(key.to_string(), vals.iter().map(|v| v.to_string()).collect());
        }
        Entry { attrs: map }
    }

    #[test]
    fn refresh_expiry_parsed_from_attr() {
        let entry = make_entry(&[("oauth2_refresh_token_expiry", &["2592000"])]);
        let data = build_advanced_data("my-app", &entry, None);
        assert_eq!(data.refresh_expiry_seconds, Some(2592000));
    }

    #[test]
    fn refresh_expiry_none_when_absent() {
        let entry = make_entry(&[]);
        let data = build_advanced_data("my-app", &entry, None);
        assert_eq!(data.refresh_expiry_seconds, None);
    }

    #[test]
    fn refresh_expiry_none_when_invalid() {
        let entry = make_entry(&[("oauth2_refresh_token_expiry", &["not-a-number"])]);
        let data = build_advanced_data("my-app", &entry, None);
        assert_eq!(data.refresh_expiry_seconds, None);
    }

    #[test]
    fn device_flow_enabled_when_true() {
        let entry = make_entry(&[("oauth2_device_flow_enable", &["true"])]);
        let data = build_advanced_data("my-app", &entry, None);
        assert!(data.device_flow_enabled);
    }

    #[test]
    fn device_flow_disabled_when_absent() {
        let entry = make_entry(&[]);
        let data = build_advanced_data("my-app", &entry, None);
        assert!(!data.device_flow_enabled);
    }

    #[test]
    fn device_flow_disabled_when_false() {
        let entry = make_entry(&[("oauth2_device_flow_enable", &["false"])]);
        let data = build_advanced_data("my-app", &entry, None);
        assert!(!data.device_flow_enabled);
    }
}

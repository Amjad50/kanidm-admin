use axum::extract::{Path, State};
use axum::response::Response;
use axum_extra::extract::Form;
use axum_htmx::HxRequest;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::friendly_client_error;
use crate::kanidm::entry::{attr_all, attr_first};

use super::common::OAuth2Kind;
use super::detail::{TabContent, compute_header, fetch_oauth2_entry, render_detail};

// ── General tab data ──────────────────────────────────────────────────────────

pub struct RedirectUrl {
    pub url: String,
    pub idx: usize,
}

pub struct GeneralData {
    pub id: String,
    pub name: String,
    pub displayname: String,
    pub landing_url: String,
    pub redirect_urls: Vec<RedirectUrl>,
    pub redirect_error: Option<String>,
    pub pkce_required: bool,
    pub pkce_disabled_for_public: bool,
    pub strict_redirect: bool,
    pub localhost_redirect: bool,
    pub is_public: bool,
    pub consent_prompt: bool,
    pub short_username: bool,
    pub legacy_crypto: bool,
    pub form_error: Option<String>,
}

// ── PKCE helper ───────────────────────────────────────────────────────────────

/// Returns `true` when PKCE is required (the default/safe state).
///
/// The kanidm attribute `oauth2_allow_insecure_client_disable_pkce` is a
/// NEGATIVE flag: when `"true"` it means PKCE is DISABLED (insecure). So:
///   - attr absent or value "false" → PKCE required → returns `true`
///   - attr value "true"            → PKCE disabled → returns `false`
pub fn pkce_required(entry: &kanidm_proto::v1::Entry) -> bool {
    !attr_first(entry, "oauth2_allow_insecure_client_disable_pkce")
        .map(|v| v == "true")
        .unwrap_or(false)
}

// ── Data builder ──────────────────────────────────────────────────────────────

fn build_general_data(
    id: &str,
    entry: &kanidm_proto::v1::Entry,
    redirect_error: Option<String>,
    form_error: Option<String>,
) -> GeneralData {
    let name = attr_first(entry, "name").unwrap_or_default();
    let displayname = attr_first(entry, "displayname")
        .or_else(|| attr_first(entry, "name"))
        .unwrap_or_default();
    let landing_url = attr_first(entry, "oauth2_rs_origin_landing").unwrap_or_default();

    let redirect_urls: Vec<RedirectUrl> = attr_all(entry, "oauth2_rs_origin")
        .into_iter()
        .enumerate()
        .map(|(idx, url)| RedirectUrl { url, idx })
        .collect();

    let is_public = matches!(super::common::detect_kind(entry), OAuth2Kind::Public);

    let pkce = pkce_required(entry);
    // Public clients always require PKCE; their toggle is read-only ON.
    let pkce_disabled_for_public = is_public;

    let strict_redirect = attr_first(entry, "oauth2_strict_redirect_uri")
        .map(|v| v == "true")
        .unwrap_or(false);

    let localhost_redirect = attr_first(entry, "oauth2_allow_localhost_redirect")
        .map(|v| v == "true")
        .unwrap_or(false);

    let consent_prompt = attr_first(entry, "oauth2_consent_prompt")
        .map(|v| v == "true")
        .unwrap_or(false);

    let short_username = attr_first(entry, "oauth2_prefer_short_username")
        .map(|v| v == "true")
        .unwrap_or(false);

    let legacy_crypto = attr_first(entry, "oauth2_jwt_legacy_crypto_enable")
        .map(|v| v == "true")
        .unwrap_or(false);

    GeneralData {
        id: id.to_string(),
        name,
        displayname,
        landing_url,
        redirect_urls,
        redirect_error,
        pkce_required: pkce,
        pkce_disabled_for_public,
        strict_redirect,
        localhost_redirect,
        is_public,
        consent_prompt,
        short_username,
        legacy_crypto,
        form_error,
    }
}

// ── Form structs ──────────────────────────────────────────────────────────────

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

#[derive(serde::Deserialize)]
pub struct AddRedirectForm {
    #[serde(default)]
    pub url: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/general
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let general_data = build_general_data(&id, &entry, None, None);
    render_detail(
        is_htmx,
        user,
        header,
        "general",
        TabContent::General(general_data),
    )
}

/// POST /oauth2/{id}/general — catch-all toggle/field update handler
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

    let enabled = form.enabled.as_deref() == Some("on");

    let result: Result<(), kanidm_client::ClientError> = match form.field.as_str() {
        "pkce" => {
            if enabled {
                client.idm_oauth2_rs_enable_pkce(&id).await
            } else {
                client.idm_oauth2_rs_disable_pkce(&id).await
            }
        }
        "strict_redirect" => {
            if enabled {
                client.idm_oauth2_rs_enable_strict_redirect_uri(&id).await
            } else {
                client.idm_oauth2_rs_disable_strict_redirect_uri(&id).await
            }
        }
        "localhost" => {
            if enabled {
                client
                    .idm_oauth2_rs_enable_public_localhost_redirect(&id)
                    .await
            } else {
                client
                    .idm_oauth2_rs_disable_public_localhost_redirect(&id)
                    .await
            }
        }
        "consent" => {
            if enabled {
                client.idm_oauth2_rs_enable_consent_prompt(&id).await
            } else {
                client.idm_oauth2_rs_disable_consent_prompt(&id).await
            }
        }
        "short_username" => {
            if enabled {
                client.idm_oauth2_rs_prefer_short_username(&id).await
            } else {
                client.idm_oauth2_rs_prefer_spn_username(&id).await
            }
        }
        "legacy_crypto" => {
            if enabled {
                client.idm_oauth2_rs_enable_legacy_crypto(&id).await
            } else {
                client.idm_oauth2_rs_disable_legacy_crypto(&id).await
            }
        }
        "name" => {
            let val = form.value.trim().to_string();
            if val.is_empty() {
                let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                let header = compute_header(&state, &entry);
                let data = build_general_data(
                    &id,
                    &entry,
                    None,
                    Some("Name cannot be empty.".to_string()),
                );
                return render_detail(is_htmx, user, header, "general", TabContent::General(data));
            }
            client
                .idm_oauth2_rs_update(&id, Some(&val), None, None, false)
                .await
        }
        "displayname" => {
            let val = form.value.trim().to_string();
            if val.is_empty() {
                let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                let header = compute_header(&state, &entry);
                let data = build_general_data(
                    &id,
                    &entry,
                    None,
                    Some("Display name cannot be empty.".to_string()),
                );
                return render_detail(is_htmx, user, header, "general", TabContent::General(data));
            }
            client
                .idm_oauth2_rs_update(&id, None, Some(&val), None, false)
                .await
        }
        "landing" => {
            let val = form.value.trim().to_string();
            if url::Url::parse(&val).is_err() {
                let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                let header = compute_header(&state, &entry);
                let data = build_general_data(
                    &id,
                    &entry,
                    None,
                    Some("Landing URL is not a valid URL.".to_string()),
                );
                return render_detail(is_htmx, user, header, "general", TabContent::General(data));
            }
            client
                .idm_oauth2_rs_update(&id, None, None, Some(&val), false)
                .await
        }
        other => {
            tracing::warn!(field = %other, "unknown general update field");
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data =
                build_general_data(&id, &entry, None, Some(format!("Unknown field: {other}")));
            return render_detail(is_htmx, user, header, "general", TabContent::General(data));
        }
    };

    // The rename case redirects to the new URL on success.
    if form.field == "name" {
        match result {
            Ok(()) => {
                let new_name = form.value.trim().to_string();
                tracing::info!(old_id = %id, new_name = %new_name, "oauth2 client renamed");
                // Re-fetch under the new name and redirect
                let entry = fetch_oauth2_entry(&state, &user, &new_name).await?;
                let header = compute_header(&state, &entry);
                let data = build_general_data(&new_name, &entry, None, None);
                return render_detail(is_htmx, user, header, "general", TabContent::General(data));
            }
            Err(e) => {
                let msg = friendly_client_error("rename oauth2 client", &e);
                let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                let header = compute_header(&state, &entry);
                let data = build_general_data(&id, &entry, None, Some(msg));
                return render_detail(is_htmx, user, header, "general", TabContent::General(data));
            }
        }
    }

    match result {
        Ok(()) => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_general_data(&id, &entry, None, None);
            render_detail(is_htmx, user, header, "general", TabContent::General(data))
        }
        Err(e) => {
            let msg = friendly_client_error("update oauth2 client", &e);
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_general_data(&id, &entry, None, Some(msg));
            render_detail(is_htmx, user, header, "general", TabContent::General(data))
        }
    }
}

/// POST /oauth2/{id}/redirect/add
pub async fn add_redirect(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<AddRedirectForm>,
) -> AppResult<Response> {
    let raw = form.url.trim().to_string();

    let parsed = match url::Url::parse(&raw) {
        Ok(u) => u,
        Err(e) => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_general_data(&id, &entry, Some(format!("Invalid URL: {e}")), None);
            return render_detail(is_htmx, user, header, "general", TabContent::General(data));
        }
    };

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client.idm_oauth2_client_add_origin(&id, &parsed).await {
        Ok(()) => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_general_data(&id, &entry, None, None);
            render_detail(is_htmx, user, header, "general", TabContent::General(data))
        }
        Err(e) => {
            let msg = friendly_client_error("add redirect URL", &e);
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_general_data(&id, &entry, Some(msg), None);
            render_detail(is_htmx, user, header, "general", TabContent::General(data))
        }
    }
}

/// POST /oauth2/{id}/redirect/{idx}/remove
pub async fn remove_redirect(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, idx)): Path<(String, usize)>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let urls = attr_all(&entry, "oauth2_rs_origin");

    let url_str = match urls.get(idx) {
        Some(u) => u.clone(),
        None => {
            let header = compute_header(&state, &entry);
            let data = build_general_data(
                &id,
                &entry,
                Some("Redirect URL not found (index out of range).".to_string()),
                None,
            );
            return render_detail(is_htmx, user, header, "general", TabContent::General(data));
        }
    };

    // URLs stored by kanidm are already valid; parse should not fail.
    let parsed = url::Url::parse(&url_str)
        .map_err(|e| AppError::Kanidm(format!("stored redirect URL is invalid: {e}")))?;

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let form_error = match client.idm_oauth2_client_remove_origin(&id, &parsed).await {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id = %id, url = %url_str, error = ?e, "remove redirect URL failed");
            Some(friendly_client_error("remove redirect URL", &e))
        }
    };

    let fresh = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &fresh);
    let data = build_general_data(&id, &fresh, form_error, None);
    render_detail(is_htmx, user, header, "general", TabContent::General(data))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use kanidm_proto::v1::Entry;
    use std::collections::BTreeMap;

    use super::pkce_required;

    fn make_entry(attrs: &[(&str, &[&str])]) -> Entry {
        let mut map = BTreeMap::new();
        for (key, vals) in attrs {
            map.insert(
                key.to_string(),
                vals.iter().map(|v| v.to_string()).collect(),
            );
        }
        Entry { attrs: map }
    }

    #[test]
    fn pkce_required_when_attr_is_true() {
        // oauth2_allow_insecure_client_disable_pkce = "true" means PKCE is disabled
        let entry = make_entry(&[("oauth2_allow_insecure_client_disable_pkce", &["true"])]);
        assert!(
            !pkce_required(&entry),
            "PKCE should NOT be required when disable flag is 'true'"
        );
    }

    #[test]
    fn pkce_required_when_attr_is_false() {
        let entry = make_entry(&[("oauth2_allow_insecure_client_disable_pkce", &["false"])]);
        assert!(
            pkce_required(&entry),
            "PKCE should be required when disable flag is 'false'"
        );
    }

    #[test]
    fn pkce_required_when_attr_absent() {
        let entry = make_entry(&[]);
        assert!(
            pkce_required(&entry),
            "PKCE should be required when disable flag is absent"
        );
    }

    #[test]
    fn pkce_required_when_attr_is_empty_string() {
        let entry = make_entry(&[("oauth2_allow_insecure_client_disable_pkce", &[""])]);
        assert!(
            pkce_required(&entry),
            "PKCE should be required when disable flag has empty value"
        );
    }
}

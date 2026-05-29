use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::Form;
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::friendly_client_error;
use crate::kanidm::entry::{attr_all, attr_first};
use crate::kanidm::scope_map::parse_scope_map;
use crate::views::partials::Modal;
use crate::AppState;

use super::detail::{compute_header, fetch_oauth2_entry, render_detail, TabContent};

// SVG icon used on scope-map modals (key icon from secret.rs)
const SCOPE_KEY_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/></svg>"#;

// ── Standard scope list ───────────────────────────────────────────────────────

pub const STANDARD_SCOPES: &[&str] = &[
    "openid",
    "profile",
    "email",
    "groups",
    "groups_name",
    "groups_spn",
    "groups_uuid",
    "ssh_publickeys",
    "offline_access",
    "roles",
];

// ── Data structs ──────────────────────────────────────────────────────────────

pub struct ScopeMapRow {
    pub group_spn: String,
    /// Short part (left of `@`) for display.
    pub group_name: String,
    /// URL-safe encoding of the SPN for use in route paths.
    pub encoded_group: String,
    /// Scopes in STANDARD_SCOPES.
    pub scopes: Vec<String>,
    /// Scopes NOT in STANDARD_SCOPES (rendered differently).
    pub custom_scopes: Vec<String>,
}

pub struct ScopeMapsData {
    pub oauth2_id: String,
    pub standard: Vec<ScopeMapRow>,
    pub supplementary: Vec<ScopeMapRow>,
    /// All group SPNs — for the `<datalist>` autocomplete.
    pub all_groups: Vec<String>,
    pub standard_scopes: &'static [&'static str],
    pub error: Option<String>,
}

// ── Modal body/footer templates ───────────────────────────────────────────────

#[derive(Template)]
#[template(path = "oauth2/_scope_map_modal_body.html")]
struct ScopeMapModalBody {
    /// "standard" or "supplementary"
    section: &'static str,
    oauth2_id: String,
    /// True = edit (group field read-only), false = add (group field editable)
    is_edit: bool,
    /// Pre-filled group SPN (edit: row's group; add: empty)
    group_spn: String,
    /// Currently active scopes for pre-checking (edit), or empty (add)
    active_scopes: Vec<String>,
    /// Pre-filled custom scopes comma-joined (edit), or empty (add)
    custom_scopes_prefill: String,
    /// All known group SPNs for datalist (add only; edit doesn't need it)
    all_groups: Vec<String>,
    /// JSON array string of SPNs that already have a scope map (for JS overwrite warning)
    existing_groups_json: String,
    standard_scopes: &'static [&'static str],
    /// Optional error to show inside modal (after a failed save)
    error: Option<String>,
}

/// Serialize a Vec<String> to a JSON array string for safe embedding in JS.
fn to_json_array(values: &[String]) -> String {
    let items: Vec<String> = values
        .iter()
        .map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    format!("[{}]", items.join(","))
}

#[derive(Template)]
#[template(path = "oauth2/_scope_map_modal_footer.html")]
struct ScopeMapModalFooter {
    /// "standard" or "supplementary"
    section: &'static str,
}

// ── Form structs ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AddScopeMapForm {
    #[serde(default)]
    pub group: String,
    /// Repeated `name="scopes"` form fields.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Optional comma-separated custom scopes appended to `scopes`.
    #[serde(default)]
    pub custom_scopes: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// URL-encode a group SPN so it is safe to embed in a route path.
/// SPNs contain `@` which must be percent-encoded.
pub(crate) fn encode_group_spn(spn: &str) -> String {
    spn.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => vec![c],
            _ => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes
                    .bytes()
                    .flat_map(|b| format!("%{b:02X}").chars().collect::<Vec<_>>())
                    .collect()
            }
        })
        .collect()
}

/// Validate a scope name: must match `^[a-z][a-z0-9_]*$`.
fn is_valid_scope(scope: &str) -> bool {
    if scope.is_empty() {
        return false;
    }
    let mut chars = scope.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Validate that a string looks like a group SPN (`name@domain`).
fn is_valid_spn(spn: &str) -> bool {
    let trimmed = spn.trim();
    if trimmed.is_empty() {
        return false;
    }
    let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
    parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
}

/// Build a `ScopeMapRow` from a raw scope map string value.
fn make_row(raw: &str) -> Option<ScopeMapRow> {
    let parsed = parse_scope_map(raw)?;
    let group_name = parsed
        .group_spn
        .split('@')
        .next()
        .unwrap_or(&parsed.group_spn)
        .to_string();
    let encoded_group = encode_group_spn(&parsed.group_spn);

    let standard_set: std::collections::HashSet<&str> = STANDARD_SCOPES.iter().copied().collect();
    let scopes: Vec<String> = parsed
        .scopes
        .iter()
        .filter(|s| standard_set.contains(s.as_str()))
        .cloned()
        .collect();
    let custom_scopes: Vec<String> = parsed
        .scopes
        .iter()
        .filter(|s| !standard_set.contains(s.as_str()))
        .cloned()
        .collect();

    Some(ScopeMapRow {
        group_spn: parsed.group_spn,
        group_name,
        encoded_group,
        scopes,
        custom_scopes,
    })
}

/// Build the full ScopeMapsData from an entry and optional error.
async fn build_scope_maps_data(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    entry: &kanidm_proto::v1::Entry,
    error: Option<String>,
) -> ScopeMapsData {
    let standard: Vec<ScopeMapRow> = attr_all(entry, "oauth2_rs_scope_map")
        .iter()
        .filter_map(|v| make_row(v))
        .collect();

    let supplementary: Vec<ScopeMapRow> = attr_all(entry, "oauth2_rs_sup_scope_map")
        .iter()
        .filter_map(|v| make_row(v))
        .collect();

    // Fetch all group SPNs for datalist autocomplete — failures are non-fatal.
    let all_groups = if let Ok(client) = state.kanidm.for_token(&user.token).await {
        client
            .idm_group_list()
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(|e| attr_first(e, "spn"))
            .collect()
    } else {
        Vec::new()
    };

    ScopeMapsData {
        oauth2_id: id.to_string(),
        standard,
        supplementary,
        all_groups,
        standard_scopes: STANDARD_SCOPES,
        error,
    }
}

/// Merge, deduplicate, and validate form scopes. Returns `Err(message)` on invalid scope.
fn build_scope_list(form: &AddScopeMapForm) -> Result<Vec<String>, String> {
    let mut scopes = form.scopes.clone();

    // Append comma-separated custom scopes.
    for s in form.custom_scopes.split(',') {
        let trimmed = s.trim().to_string();
        if !trimmed.is_empty() {
            scopes.push(trimmed);
        }
    }

    // Validate each scope.
    for scope in &scopes {
        if !is_valid_scope(scope) {
            return Err(format!(
                "Scope '{scope}' is not valid: scopes must start with a lowercase letter and contain only lowercase letters, digits, and underscores."
            ));
        }
    }

    // Deduplicate preserving order.
    let mut seen = std::collections::HashSet::new();
    scopes.retain(|s| seen.insert(s.clone()));

    Ok(scopes)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/scope-maps
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;
    render_detail(is_htmx, user, header, "scope-maps", TabContent::ScopeMaps(data))
}

/// Build and render a scope-map modal (add or edit).
fn render_scope_map_modal(
    title: String,
    section: &'static str,
    oauth2_id: String,
    is_edit: bool,
    group_spn: String,
    active_scopes: Vec<String>,
    custom_scopes_prefill: String,
    all_groups: Vec<String>,
    existing_groups: Vec<String>,
    error: Option<String>,
) -> AppResult<String> {
    let existing_groups_json = to_json_array(&existing_groups);
    let body_html = ScopeMapModalBody {
        section,
        oauth2_id: oauth2_id.clone(),
        is_edit,
        group_spn,
        active_scopes,
        custom_scopes_prefill,
        all_groups,
        existing_groups_json,
        standard_scopes: STANDARD_SCOPES,
        error,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = ScopeMapModalFooter { section }
        .render()
        .map_err(AppError::Template)?;

    Modal {
        title,
        icon_svg: Some(SCOPE_KEY_SVG),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)
}

/// Shared implementation for adding a standard or supplementary scope map.
/// On success: re-renders tab fragment + OOB modal-close.
/// On validation/API error with HTMX: re-renders modal with inline error.
async fn add_scope_map_impl(
    state: AppState,
    is_htmx: bool,
    Path(id): Path<String>,
    user: AdminUser,
    form: AddScopeMapForm,
    supplementary: bool,
) -> AppResult<Response> {
    let section: &'static str = if supplementary { "supplementary" } else { "standard" };
    let group = form.group.trim().to_string();

    // Validate group SPN.
    if !is_valid_spn(&group) {
        let err_msg = "Group must be a valid SPN (e.g. my-group@domain.example).".to_string();
        if is_htmx {
            // Re-render the Add modal with error inline.
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;
            let existing_groups: Vec<String> = if supplementary {
                data.supplementary.iter().map(|r| r.group_spn.clone()).collect()
            } else {
                data.standard.iter().map(|r| r.group_spn.clone()).collect()
            };
            let modal_title = format!(
                "Add {} scope map",
                if supplementary { "supplementary" } else { "standard" }
            );
            let html = render_scope_map_modal(
                modal_title,
                section,
                id,
                false,
                group,
                form.scopes.clone(),
                form.custom_scopes.clone(),
                data.all_groups,
                existing_groups,
                Some(err_msg),
            )?;
            return Ok(Html(html).into_response());
        }
        let entry = fetch_oauth2_entry(&state, &user, &id).await?;
        let header = compute_header(&state, &entry);
        let data = build_scope_maps_data(&state, &user, &id, &entry, Some(err_msg)).await;
        return render_detail(is_htmx, user, header, "scope-maps", TabContent::ScopeMaps(data));
    }

    // Build and validate scopes.
    let scopes = match build_scope_list(&form) {
        Ok(s) => s,
        Err(msg) => {
            if is_htmx {
                let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;
                let existing_groups: Vec<String> = if supplementary {
                    data.supplementary.iter().map(|r| r.group_spn.clone()).collect()
                } else {
                    data.standard.iter().map(|r| r.group_spn.clone()).collect()
                };
                let modal_title = format!(
                    "Add {} scope map",
                    if supplementary { "supplementary" } else { "standard" }
                );
                let html = render_scope_map_modal(
                    modal_title,
                    section,
                    id,
                    false,
                    group,
                    form.scopes.clone(),
                    form.custom_scopes.clone(),
                    data.all_groups,
                    existing_groups,
                    Some(msg),
                )?;
                return Ok(Html(html).into_response());
            }
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_scope_maps_data(&state, &user, &id, &entry, Some(msg)).await;
            return render_detail(
                is_htmx,
                user,
                header,
                "scope-maps",
                TabContent::ScopeMaps(data),
            );
        }
    };

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let scope_refs: Vec<&str> = scopes.iter().map(String::as_str).collect();
    let result = if supplementary {
        client
            .idm_oauth2_rs_update_sup_scope_map(&id, &group, scope_refs)
            .await
    } else {
        client
            .idm_oauth2_rs_update_scope_map(&id, &group, scope_refs)
            .await
    };

    match result {
        Ok(()) => {
            // Success: close modal (primary overlay-slot target becomes empty),
            // and OOB-swap the tab content + nav.
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;
            if is_htmx {
                let tab_html = {
                    use askama::Template;
                    crate::handlers::oauth2::detail::TabContentFragment {
                        tab_content: &TabContent::ScopeMaps(data),
                        header: &header,
                    }
                    .render()
                    .map_err(AppError::Template)?
                };
                let nav_html = {
                    use askama::Template;
                    crate::handlers::oauth2::detail::TabsNavFragment {
                        header: &header,
                        active_tab: "scope-maps",
                        tabs: crate::handlers::oauth2::detail::TABS,
                        oob: true,
                    }
                    .render()
                    .map_err(AppError::Template)?
                };
                // Primary response: empty (clears overlay-slot / closes modal).
                // OOB: tab-content update + nav update.
                let tab_oob = format!(
                    r#"<div id="tab-content" hx-swap-oob="innerHTML">{tab_html}</div>"#
                );
                return Ok(Html(format!("{tab_oob}{nav_html}")).into_response());
            }
            render_detail(is_htmx, user, header, "scope-maps", TabContent::ScopeMaps(data))
        }
        Err(e) => {
            tracing::warn!(id = %id, group = %group, supplementary, error = ?e, "add scope map failed");
            let err_msg = friendly_client_error("add scope map", &e);
            if is_htmx {
                // Re-render modal with error.
                let entry = fetch_oauth2_entry(&state, &user, &id).await?;
                let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;
                let existing_groups: Vec<String> = if supplementary {
                    data.supplementary.iter().map(|r| r.group_spn.clone()).collect()
                } else {
                    data.standard.iter().map(|r| r.group_spn.clone()).collect()
                };
                let modal_title = format!(
                    "Add {} scope map",
                    if supplementary { "supplementary" } else { "standard" }
                );
                let html = render_scope_map_modal(
                    modal_title,
                    section,
                    id,
                    false,
                    group,
                    scopes,
                    form.custom_scopes,
                    data.all_groups,
                    existing_groups,
                    Some(err_msg),
                )?;
                return Ok(Html(html).into_response());
            }
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_scope_maps_data(&state, &user, &id, &entry, Some(err_msg)).await;
            render_detail(is_htmx, user, header, "scope-maps", TabContent::ScopeMaps(data))
        }
    }
}

/// POST /oauth2/{id}/scope-map/standard
pub async fn add_standard(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<AddScopeMapForm>,
) -> AppResult<Response> {
    add_scope_map_impl(state, is_htmx, Path(id), user, form, false).await
}

/// POST /oauth2/{id}/scope-map/standard/{group}/delete
pub async fn delete_standard(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, group)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let error = match client.idm_oauth2_rs_delete_scope_map(&id, &group).await {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id = %id, group = %group, error = ?e, "delete standard scope map failed");
            Some(friendly_client_error("delete scope map", &e))
        }
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_scope_maps_data(&state, &user, &id, &entry, error).await;
    render_detail(is_htmx, user, header, "scope-maps", TabContent::ScopeMaps(data))
}

/// POST /oauth2/{id}/scope-map/supplementary
pub async fn add_supplementary(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<AddScopeMapForm>,
) -> AppResult<Response> {
    add_scope_map_impl(state, is_htmx, Path(id), user, form, true).await
}

/// POST /oauth2/{id}/scope-map/supplementary/{group}/delete
pub async fn delete_supplementary(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, group)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let error = match client
        .idm_oauth2_rs_delete_sup_scope_map(&id, &group)
        .await
    {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id = %id, group = %group, error = ?e, "delete supplementary scope map failed");
            Some(friendly_client_error("delete scope map", &e))
        }
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_scope_maps_data(&state, &user, &id, &entry, error).await;
    render_detail(is_htmx, user, header, "scope-maps", TabContent::ScopeMaps(data))
}

/// Shared helper for rendering the Add modal (new scope map entry).
async fn new_modal_impl(
    state: AppState,
    Path(id): Path<String>,
    user: AdminUser,
    supplementary: bool,
) -> AppResult<Response> {
    let section: &'static str = if supplementary { "supplementary" } else { "standard" };
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;

    let existing_groups: Vec<String> = if supplementary {
        data.supplementary.iter().map(|r| r.group_spn.clone()).collect()
    } else {
        data.standard.iter().map(|r| r.group_spn.clone()).collect()
    };

    let modal_title = format!(
        "Add {} scope map",
        if supplementary { "supplementary" } else { "standard" }
    );

    let html = render_scope_map_modal(
        modal_title,
        section,
        id,
        false,
        String::new(),
        vec![],
        String::new(),
        data.all_groups,
        existing_groups,
        None,
    )?;
    Ok(Html(html).into_response())
}

/// Shared helper for rendering the Edit modal (pre-populated with row data).
async fn edit_modal_impl(
    state: AppState,
    Path((id, group)): Path<(String, String)>,
    user: AdminUser,
    supplementary: bool,
) -> AppResult<Response> {
    let section: &'static str = if supplementary { "supplementary" } else { "standard" };
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let data = build_scope_maps_data(&state, &user, &id, &entry, None).await;

    // Find the row being edited.
    let rows = if supplementary { &data.supplementary } else { &data.standard };
    let row = rows.iter().find(|r| r.group_spn == group || r.encoded_group == group);

    let (active_scopes, custom_scopes_prefill) = if let Some(r) = row {
        (r.scopes.clone(), r.custom_scopes.join(", "))
    } else {
        (vec![], String::new())
    };

    let modal_title = format!(
        "Edit {} scope map — {}",
        if supplementary { "supplementary" } else { "standard" },
        group
    );

    let html = render_scope_map_modal(
        modal_title,
        section,
        id,
        true,
        group,
        active_scopes,
        custom_scopes_prefill,
        vec![],      // no datalist needed for edit
        vec![],      // no overwrite warning needed for edit
        None,
    )?;
    Ok(Html(html).into_response())
}

/// GET /oauth2/{id}/scope-map/standard/new
pub async fn standard_new_modal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    new_modal_impl(state, Path(id), user, false).await
}

/// GET /oauth2/{id}/scope-map/standard/{group}/edit
pub async fn standard_edit_modal(
    State(state): State<AppState>,
    Path((id, group)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    edit_modal_impl(state, Path((id, group)), user, false).await
}

/// GET /oauth2/{id}/scope-map/supplementary/new
pub async fn supplementary_new_modal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    new_modal_impl(state, Path(id), user, true).await
}

/// GET /oauth2/{id}/scope-map/supplementary/{group}/edit
pub async fn supplementary_edit_modal(
    State(state): State<AppState>,
    Path((id, group)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    edit_modal_impl(state, Path((id, group)), user, true).await
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{encode_group_spn, is_valid_scope, is_valid_spn};

    #[test]
    fn scope_valid_simple() {
        assert!(is_valid_scope("openid"));
        assert!(is_valid_scope("email"));
        assert!(is_valid_scope("offline_access"));
        assert!(is_valid_scope("groups_name"));
    }

    #[test]
    fn scope_invalid_starts_with_digit() {
        assert!(!is_valid_scope("1scope"));
    }

    #[test]
    fn scope_invalid_uppercase() {
        assert!(!is_valid_scope("OpenID"));
    }

    #[test]
    fn scope_invalid_empty() {
        assert!(!is_valid_scope(""));
    }

    #[test]
    fn spn_valid() {
        assert!(is_valid_spn("group@domain.example"));
    }

    #[test]
    fn spn_invalid_no_at() {
        assert!(!is_valid_spn("nodomainsep"));
    }

    #[test]
    fn spn_invalid_empty() {
        assert!(!is_valid_spn(""));
    }

    #[test]
    fn encode_at_sign() {
        assert_eq!(encode_group_spn("group@domain"), "group%40domain");
    }
}

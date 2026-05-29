use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::Form;
use axum_htmx::HxRequest;
use kanidm_proto::internal::Oauth2ClaimMapJoin;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::friendly_client_error;
use crate::handlers::oauth2::scope_maps::encode_group_spn;
use crate::kanidm::claim_map::parse_claim_map;
use crate::kanidm::entry::{attr_all, attr_first};
use crate::views::partials::Modal;
use crate::AppState;

use super::detail::{compute_header, fetch_oauth2_entry, render_detail, TabContent};

// SVG icon for claim map modals (tag/label icon fits "claims")
const CLAIM_TAG_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>"#;

// ── Data structs ──────────────────────────────────────────────────────────────

pub struct ClaimMapRow {
    pub group_spn: String,
    /// Short part (left of `@`) for display.
    pub group_name: String,
    /// URL-safe encoding of the group SPN for route paths.
    pub encoded_group: String,
    pub values: Vec<String>,
    /// Pre-joined comma-separated display string.
    pub values_csv: String,
}

pub struct ClaimGroupView {
    pub claim_name: String,
    pub current_join: Oauth2ClaimMapJoin,
    /// Human-readable label: "csv", "ssv", or "array".
    pub current_join_label: &'static str,
    /// Short description of the join strategy.
    pub current_join_desc: &'static str,
    pub rows: Vec<ClaimMapRow>,
}

pub struct ClaimMapsData {
    pub oauth2_id: String,
    /// Claim groups sorted by claim name.
    pub claims: Vec<ClaimGroupView>,
    /// All group SPNs for datalist autocomplete.
    pub all_groups: Vec<String>,
    pub error: Option<String>,
}

// ── Modal template structs ────────────────────────────────────────────────────

/// Body for the "Add new claim" modal (claim name editable, join editable, first group+values).
#[derive(Template)]
#[template(path = "oauth2/_claim_map_new_modal_body.html")]
struct ClaimNewModalBody {
    oauth2_id: String,
    all_groups: Vec<String>,
    error: Option<String>,
}

/// Body for edit-row and add-group-to-claim modals.
#[derive(Template)]
#[template(path = "oauth2/_claim_map_row_modal_body.html")]
struct ClaimRowModalBody {
    oauth2_id: String,
    /// True = editing existing row (claim+group read-only), false = adding a group to an existing claim.
    is_edit: bool,
    claim_name: String,
    group_spn: String,
    /// Pre-filled comma-separated values string.
    values_prefill: String,
    /// Current join label for display (shown but disabled when is_edit).
    join_label: &'static str,
    all_groups: Vec<String>,
    error: Option<String>,
}

/// Body for the "Change join strategy" modal.
#[derive(Template)]
#[template(path = "oauth2/_claim_map_join_modal_body.html")]
struct ClaimJoinModalBody {
    oauth2_id: String,
    claim_name: String,
    current_join_label: &'static str,
    error: Option<String>,
}

/// Shared footer for all claim-map modals.
#[derive(Template)]
#[template(path = "oauth2/_claim_map_modal_footer.html")]
struct ClaimModalFooter {
    cancel_only: bool,
}

// ── Form structs ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AddClaimMapForm {
    #[serde(default)]
    pub claim: String,
    #[serde(default)]
    pub group: String,
    /// Comma-separated user input.
    #[serde(default)]
    pub values: String,
    /// "csv" | "ssv" | "array"
    #[serde(default)]
    pub join: String,
}

#[derive(serde::Deserialize)]
pub struct SetJoinForm {
    #[serde(default)]
    pub join: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn join_label(join: &Oauth2ClaimMapJoin) -> &'static str {
    match join {
        Oauth2ClaimMapJoin::Csv => "csv",
        Oauth2ClaimMapJoin::Ssv => "ssv",
        Oauth2ClaimMapJoin::Array => "array",
    }
}

fn join_desc(join: &Oauth2ClaimMapJoin) -> &'static str {
    match join {
        Oauth2ClaimMapJoin::Csv => "Emitted as a comma-separated string.",
        Oauth2ClaimMapJoin::Ssv => "Emitted as a space-separated string.",
        Oauth2ClaimMapJoin::Array => "Emitted as a JSON array.",
    }
}

fn parse_join_str(s: &str) -> Option<Oauth2ClaimMapJoin> {
    match s {
        "csv" => Some(Oauth2ClaimMapJoin::Csv),
        "ssv" => Some(Oauth2ClaimMapJoin::Ssv),
        "array" => Some(Oauth2ClaimMapJoin::Array),
        _ => None,
    }
}

/// Validate a claim name: must match `^[a-z][a-z0-9_]*$`.
fn is_valid_claim_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
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

/// Build the full `ClaimMapsData` from an entry and optional error.
async fn build_claim_maps_data(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    entry: &kanidm_proto::v1::Entry,
    error: Option<String>,
) -> ClaimMapsData {
    // Parse all claim map values and group by claim name.
    let raw_values = attr_all(entry, "oauth2_rs_claim_map");

    // Group by claim name, preserving insertion order via a Vec + linear scan.
    let mut claims: Vec<ClaimGroupView> = Vec::new();

    for raw in &raw_values {
        let parsed = match parse_claim_map(raw) {
            Some(p) => p,
            None => {
                tracing::warn!(raw = %raw, "failed to parse oauth2_rs_claim_map value");
                continue;
            }
        };

        let group_name = parsed
            .group_spn
            .split('@')
            .next()
            .unwrap_or(&parsed.group_spn)
            .to_string();
        let encoded_group = encode_group_spn(&parsed.group_spn);
        let values_csv = parsed.values.join(", ");

        let row = ClaimMapRow {
            group_spn: parsed.group_spn,
            group_name,
            encoded_group,
            values: parsed.values,
            values_csv,
        };

        if let Some(group_view) = claims.iter_mut().find(|g| g.claim_name == parsed.claim_name) {
            // Update join strategy — last value wins (all rows for a claim share the same join).
            group_view.current_join = parsed.join;
            group_view.current_join_label = join_label(&parsed.join);
            group_view.current_join_desc = join_desc(&parsed.join);
            group_view.rows.push(row);
        } else {
            claims.push(ClaimGroupView {
                claim_name: parsed.claim_name.clone(),
                current_join: parsed.join,
                current_join_label: join_label(&parsed.join),
                current_join_desc: join_desc(&parsed.join),
                rows: vec![row],
            });
        }
    }

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

    ClaimMapsData {
        oauth2_id: id.to_string(),
        claims,
        all_groups,
        error,
    }
}

// ── Modal/OOB helpers ─────────────────────────────────────────────────────────

/// Render the claim-maps tab as OOB + nav OOB, primary response empty (closes modal).
fn render_claim_tab_with_oob_close(
    header: &crate::handlers::oauth2::detail::OAuth2Header,
    data: ClaimMapsData,
) -> AppResult<String> {
    let tab_html = {
        crate::handlers::oauth2::detail::TabContentFragment {
            tab_content: &TabContent::ClaimMaps(data),
            header,
        }
        .render()
        .map_err(AppError::Template)?
    };
    let nav_html = {
        crate::handlers::oauth2::detail::TabsNavFragment {
            header,
            active_tab: "claim-maps",
            tabs: crate::handlers::oauth2::detail::TABS,
            oob: true,
        }
        .render()
        .map_err(AppError::Template)?
    };
    // Primary target (#overlay-slot) gets cleared; tab-content and nav OOB.
    Ok(format!(
        r#"<div id="tab-content" hx-swap-oob="innerHTML">{tab_html}</div>{nav_html}"#
    ))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/claim-maps
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_claim_maps_data(&state, &user, &id, &entry, None).await;
    render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data))
}

/// POST /oauth2/{id}/claim-map  — Add a (claim, group) row with values + initial join.
pub async fn add(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<AddClaimMapForm>,
) -> AppResult<Response> {
    let claim = form.claim.trim().to_string();
    let group = form.group.trim().to_string();

    // Validate claim name.
    if !is_valid_claim_name(&claim) {
        let entry = fetch_oauth2_entry(&state, &user, &id).await?;
        let header = compute_header(&state, &entry);
        let data = build_claim_maps_data(
            &state,
            &user,
            &id,
            &entry,
            Some("Claim name must start with a lowercase letter and contain only lowercase letters, digits, and underscores.".to_string()),
        )
        .await;
        return render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data));
    }

    // Validate group SPN.
    if !is_valid_spn(&group) {
        let entry = fetch_oauth2_entry(&state, &user, &id).await?;
        let header = compute_header(&state, &entry);
        let data = build_claim_maps_data(
            &state,
            &user,
            &id,
            &entry,
            Some("Group must be a valid SPN (e.g. my-group@domain.example).".to_string()),
        )
        .await;
        return render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data));
    }

    // Parse and validate values.
    let values: Vec<String> = form
        .values
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if values.is_empty() {
        let entry = fetch_oauth2_entry(&state, &user, &id).await?;
        let header = compute_header(&state, &entry);
        let data = build_claim_maps_data(
            &state,
            &user,
            &id,
            &entry,
            Some("Values must not be empty. Enter at least one value.".to_string()),
        )
        .await;
        return render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data));
    }

    // Validate join strategy.
    let join = match parse_join_str(&form.join) {
        Some(j) => j,
        None => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_claim_maps_data(
                &state,
                &user,
                &id,
                &entry,
                Some("Join strategy must be one of: csv, ssv, array.".to_string()),
            )
            .await;
            return render_detail(
                is_htmx,
                user,
                header,
                "claim-maps",
                TabContent::ClaimMaps(data),
            );
        }
    };

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    // Set join strategy first (so it's in place before values land).
    let join_result = client
        .idm_oauth2_rs_update_claim_map_join(&id, &claim, join)
        .await;
    let error = match join_result {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id = %id, claim = %claim, error = ?e, "set claim map join failed");
            Some(friendly_client_error("add claim map", &e))
        }
    };

    // Only proceed with values if the join succeeded.
    let error = if error.is_none() {
        let values_result = client
            .idm_oauth2_rs_update_claim_map(&id, &claim, &group, &values)
            .await;
        match values_result {
            Ok(()) => None,
            Err(e) => {
                tracing::warn!(id = %id, claim = %claim, group = %group, error = ?e, "add claim map failed");
                Some(friendly_client_error("add claim map", &e))
            }
        }
    } else {
        error
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_claim_maps_data(&state, &user, &id, &entry, error.clone()).await;
    if is_htmx && error.is_none() {
        // Success from modal: close modal + OOB tab update.
        let html = render_claim_tab_with_oob_close(&header, data)?;
        return Ok(Html(html).into_response());
    }
    render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data))
}

/// POST /oauth2/{id}/claim-map/{claim}/{group}/delete
pub async fn delete(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, claim, group)): Path<(String, String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    // The group path segment is percent-encoded; axum decodes it automatically.
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let error = match client
        .idm_oauth2_rs_delete_claim_map(&id, &claim, &group)
        .await
    {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id = %id, claim = %claim, group = %group, error = ?e, "delete claim map failed");
            Some(friendly_client_error("delete claim map", &e))
        }
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_claim_maps_data(&state, &user, &id, &entry, error).await;
    render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data))
}

/// POST /oauth2/{id}/claim-map/{claim}/join  — Change the join strategy for a claim.
pub async fn set_join(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, claim)): Path<(String, String)>,
    user: AdminUser,
    Form(form): Form<SetJoinForm>,
) -> AppResult<Response> {
    let join = match parse_join_str(&form.join) {
        Some(j) => j,
        None => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_claim_maps_data(
                &state,
                &user,
                &id,
                &entry,
                Some("Join strategy must be one of: csv, ssv, array.".to_string()),
            )
            .await;
            return render_detail(
                is_htmx,
                user,
                header,
                "claim-maps",
                TabContent::ClaimMaps(data),
            );
        }
    };

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let error = match client
        .idm_oauth2_rs_update_claim_map_join(&id, &claim, join)
        .await
    {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id = %id, claim = %claim, error = ?e, "set claim map join failed");
            Some(friendly_client_error("set claim map join", &e))
        }
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_claim_maps_data(&state, &user, &id, &entry, error.clone()).await;
    if is_htmx && error.is_none() {
        // Success from join-strategy modal: close modal + OOB tab update.
        let html = render_claim_tab_with_oob_close(&header, data)?;
        return Ok(Html(html).into_response());
    }
    render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data))
}

// ── New modal GET handlers ────────────────────────────────────────────────────

/// GET /oauth2/{id}/claim-map/new  — "Add new claim" modal.
pub async fn new_claim_modal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let data = build_claim_maps_data(&state, &user, &id, &entry, None).await;

    let body_html = ClaimNewModalBody {
        oauth2_id: id.clone(),
        all_groups: data.all_groups,
        error: None,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = ClaimModalFooter { cancel_only: false }
        .render()
        .map_err(AppError::Template)?;

    let html = Modal {
        title: "Add new claim".to_string(),
        icon_svg: Some(CLAIM_TAG_SVG),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

/// GET /oauth2/{id}/claim-map/{claim}/{group}/edit  — Edit an existing (claim, group) row.
pub async fn edit_row_modal(
    State(state): State<AppState>,
    Path((id, claim, group)): Path<(String, String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let data = build_claim_maps_data(&state, &user, &id, &entry, None).await;

    // Find the existing row to pre-fill values and join.
    let (values_prefill, join_lbl) = if let Some(claim_view) =
        data.claims.iter().find(|c| c.claim_name == claim)
    {
        let row_vals = claim_view
            .rows
            .iter()
            .find(|r| r.group_spn == group || r.encoded_group == group)
            .map(|r| r.values_csv.clone())
            .unwrap_or_default();
        (row_vals, claim_view.current_join_label)
    } else {
        (String::new(), "csv")
    };

    let body_html = ClaimRowModalBody {
        oauth2_id: id.clone(),
        is_edit: true,
        claim_name: claim.clone(),
        group_spn: group.clone(),
        values_prefill,
        join_label: join_lbl,
        all_groups: vec![],
        error: None,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = ClaimModalFooter { cancel_only: false }
        .render()
        .map_err(AppError::Template)?;

    let html = Modal {
        title: format!("Edit claim map — {claim} ({group})"),
        icon_svg: Some(CLAIM_TAG_SVG),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

/// GET /oauth2/{id}/claim-map/{claim}/add-group  — Add a new group to an existing claim.
pub async fn add_group_modal(
    State(state): State<AppState>,
    Path((id, claim)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let data = build_claim_maps_data(&state, &user, &id, &entry, None).await;

    let join_lbl = data
        .claims
        .iter()
        .find(|c| c.claim_name == claim)
        .map(|c| c.current_join_label)
        .unwrap_or("csv");

    let body_html = ClaimRowModalBody {
        oauth2_id: id.clone(),
        is_edit: false,
        claim_name: claim.clone(),
        group_spn: String::new(),
        values_prefill: String::new(),
        join_label: join_lbl,
        all_groups: data.all_groups,
        error: None,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = ClaimModalFooter { cancel_only: false }
        .render()
        .map_err(AppError::Template)?;

    let html = Modal {
        title: format!("Add group to {claim}"),
        icon_svg: Some(CLAIM_TAG_SVG),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

/// GET /oauth2/{id}/claim-map/{claim}/join-modal  — Change join strategy modal.
pub async fn join_strategy_modal(
    State(state): State<AppState>,
    Path((id, claim)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let data = build_claim_maps_data(&state, &user, &id, &entry, None).await;

    let current_join_lbl = data
        .claims
        .iter()
        .find(|c| c.claim_name == claim)
        .map(|c| c.current_join_label)
        .unwrap_or("csv");

    let body_html = ClaimJoinModalBody {
        oauth2_id: id.clone(),
        claim_name: claim.clone(),
        current_join_label: current_join_lbl,
        error: None,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = ClaimModalFooter { cancel_only: false }
        .render()
        .map_err(AppError::Template)?;

    let html = Modal {
        title: format!("Join strategy for {claim}"),
        icon_svg: Some(CLAIM_TAG_SVG),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-sm",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

/// POST /oauth2/{id}/claim-map/{claim}/delete-all  — Delete every group entry for a claim.
pub async fn delete_all_for_claim(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, claim)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    // Fetch entry to get current rows for this claim.
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let raw_values = attr_all(&entry, "oauth2_rs_claim_map");

    // Collect (claim, group) pairs that belong to this claim.
    let mut pairs: Vec<(String, String)> = Vec::new();
    for raw in &raw_values {
        if let Some(parsed) = parse_claim_map(raw) {
            if parsed.claim_name == claim {
                pairs.push((parsed.claim_name, parsed.group_spn));
            }
        }
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let mut errors: Vec<String> = Vec::new();
    for (claim_name, group_spn) in &pairs {
        if let Err(e) = client
            .idm_oauth2_rs_delete_claim_map(&id, claim_name, group_spn)
            .await
        {
            tracing::warn!(
                id = %id,
                claim = %claim_name,
                group = %group_spn,
                error = ?e,
                "delete claim row failed during delete-all"
            );
            errors.push(friendly_client_error("delete claim map", &e));
        }
    }

    let combined_error = if errors.is_empty() {
        None
    } else {
        Some(format!("Some rows could not be deleted: {}", errors.join("; ")))
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_claim_maps_data(&state, &user, &id, &entry, combined_error).await;
    render_detail(is_htmx, user, header, "claim-maps", TabContent::ClaimMaps(data))
}

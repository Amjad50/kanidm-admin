use axum::extract::{Path, State};
use axum::response::Response;
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::kanidm::entry::attr_all;
use crate::kanidm::key_state::parse_key_state;
use crate::kanidm::scope_map::parse_scope_map;
use crate::kanidm::claim_map::parse_claim_map;
use crate::AppState;

use super::detail::{compute_header, fetch_oauth2_entry, render_detail, TabContent};
use super::general::pkce_required;
use crate::kanidm::entry::attr_first;

// ── Data structs ──────────────────────────────────────────────────────────────

pub struct OverviewToggles {
    pub pkce_required: bool,
    pub strict_redirect: bool,
    pub localhost_redirects: bool,
    pub consent_prompt: bool,
    pub prefer_short_username: bool,
    pub legacy_crypto: bool,
}

pub struct ScopeMapSummary {
    pub group_name: String,
    pub group_spn: String,
    pub scopes: Vec<String>,
}

pub struct ClaimMapSummary {
    pub claim_name: String,
    pub group_name: String,
    pub values_csv: String,
}

pub struct KeyRowSummary {
    pub id: String,
    pub status_label: &'static str,
    pub status_badge_classes: &'static str,
    pub algorithm: String,
}

pub struct OverviewData {
    pub oauth2_id: String,
    pub uuid: String,
    pub displayname: String,
    pub name: String,
    pub kind_label: &'static str,
    pub kind_badge_classes: &'static str,
    pub landing_url: String,
    pub supplementary_redirects: Vec<String>,
    pub toggles: OverviewToggles,
    pub standard_scope_maps: Vec<ScopeMapSummary>,
    pub supplementary_scope_maps: Vec<ScopeMapSummary>,
    pub claim_map_summaries: Vec<ClaimMapSummary>,
    pub keys: Vec<KeyRowSummary>,
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Extract group short name (left of '@', or the full SPN if no '@' found).
fn short_group_name(spn: &str) -> String {
    spn.split('@').next().unwrap_or(spn).to_string()
}

pub(super) fn build_overview_data(
    id: &str,
    entry: &kanidm_proto::v1::Entry,
    header: &super::detail::OAuth2Header,
) -> OverviewData {
    let uuid = attr_first(entry, "uuid").unwrap_or_default();
    let displayname = header.displayname.clone();
    let name = header.name.clone();

    let landing_url = attr_first(entry, "oauth2_rs_origin_landing").unwrap_or_default();
    let supplementary_redirects = attr_all(entry, "oauth2_rs_origin");

    // Toggles
    let strict_redirect = attr_first(entry, "oauth2_strict_redirect_uri")
        .map(|v| v == "true")
        .unwrap_or(false);
    let localhost_redirects = attr_first(entry, "oauth2_allow_localhost_redirect")
        .map(|v| v == "true")
        .unwrap_or(false);
    let consent_prompt = attr_first(entry, "oauth2_consent_prompt")
        .map(|v| v == "true")
        .unwrap_or(false);
    let prefer_short_username = attr_first(entry, "oauth2_prefer_short_username")
        .map(|v| v == "true")
        .unwrap_or(false);
    let legacy_crypto = attr_first(entry, "oauth2_jwt_legacy_crypto_enable")
        .map(|v| v == "true")
        .unwrap_or(false);

    let toggles = OverviewToggles {
        pkce_required: pkce_required(entry),
        strict_redirect,
        localhost_redirects,
        consent_prompt,
        prefer_short_username,
        legacy_crypto,
    };

    // Scope maps
    let standard_scope_maps: Vec<ScopeMapSummary> = attr_all(entry, "oauth2_rs_scope_map")
        .into_iter()
        .filter_map(|raw| parse_scope_map(&raw))
        .map(|m| ScopeMapSummary {
            group_name: short_group_name(&m.group_spn),
            group_spn: m.group_spn,
            scopes: m.scopes,
        })
        .collect();

    let supplementary_scope_maps: Vec<ScopeMapSummary> =
        attr_all(entry, "oauth2_rs_sup_scope_map")
            .into_iter()
            .filter_map(|raw| parse_scope_map(&raw))
            .map(|m| ScopeMapSummary {
                group_name: short_group_name(&m.group_spn),
                group_spn: m.group_spn,
                scopes: m.scopes,
            })
            .collect();

    // Claim maps
    let claim_map_summaries: Vec<ClaimMapSummary> = attr_all(entry, "oauth2_rs_claim_map")
        .into_iter()
        .filter_map(|raw| parse_claim_map(&raw))
        .map(|m| {
            let group_name = short_group_name(&m.group_spn);
            let values_csv = m.values.join(", ");
            ClaimMapSummary {
                claim_name: m.claim_name,
                group_name,
                values_csv,
            }
        })
        .collect();

    // Keys
    let keys: Vec<KeyRowSummary> = {
        let mut rows: Vec<_> = attr_all(entry, "key_internal_data")
            .into_iter()
            .filter_map(|raw| parse_key_state(&raw))
            .collect();
        rows.sort_by_key(|k| k.status.sort_order());
        rows.into_iter()
            .map(|k| KeyRowSummary {
                id: k.id,
                status_label: k.status.label(),
                status_badge_classes: k.status.badge_classes(),
                algorithm: k.algorithm,
            })
            .collect()
    };

    OverviewData {
        oauth2_id: id.to_string(),
        uuid,
        displayname,
        name,
        kind_label: header.kind_label,
        kind_badge_classes: header.kind_badge_classes,
        landing_url,
        supplementary_redirects,
        toggles,
        standard_scope_maps,
        supplementary_scope_maps,
        claim_map_summaries,
        keys,
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/overview
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let overview_data = build_overview_data(&id, &entry, &header);
    render_detail(is_htmx, user, header, "overview", TabContent::Overview(overview_data))
}

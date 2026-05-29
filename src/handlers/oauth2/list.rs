use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::{wants_json, PaletteItem, PaletteResponse};
use crate::kanidm::entry::{attr_all, attr_first, attr_present};
use crate::views::{initials, BaseFields};
use crate::AppState;

use super::common::{detect_kind, OAuth2Kind};

// ── Row data ──────────────────────────────────────────────────────────────────

pub struct OAuth2AppRow {
    pub name: String,
    pub displayname: String,
    pub initials: String,
    pub image_url: Option<String>,
    pub kind: OAuth2Kind,
    pub landing_url: Option<String>,
    pub detail_href: String,
    /// Pre-rendered actions cell HTML (single icon button or kebab + dropdown).
    pub actions_html: String,
}

fn build_app_actions(name: &str, displayname: &str, kind: OAuth2Kind) -> String {
    use crate::views::dropdown::{render_actions_cell, DropdownItem};
    let mut items: Vec<DropdownItem> = Vec::new();
    if matches!(kind, OAuth2Kind::Basic) {
        items.push(
            DropdownItem::htmx_get("View secret", format!("/oauth2/{name}/secret"))
                .with_icon("key"),
        );
    }
    items.push(
        DropdownItem::link("Scope maps", format!("/oauth2/{name}/scope-maps"))
            .with_icon("users"),
    );
    items.push(DropdownItem::Divider);
    items.push(
        DropdownItem::htmx_get("Delete", format!("/oauth2/{name}/delete"))
            .with_icon("trash-2")
            .danger(),
    );
    render_actions_cell(items, format!("Actions for {displayname}"))
}

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct ListParams {
    pub q: Option<String>,
    pub page: Option<usize>,
    pub per: Option<usize>,
}

// ── View structs ──────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "oauth2/list.html")]
pub struct OAuth2ListView {
    pub base: BaseFields,
    pub apps: Vec<OAuth2AppRow>,
    pub q: String,
    pub per: usize,
    pub pagination: crate::views::pagination::Pagination,
    pub count_text: String,
}

#[derive(Template)]
#[template(path = "oauth2/_cards.html")]
pub struct OAuth2CardsFragment {
    pub apps: Vec<OAuth2AppRow>,
    pub q: String,
}

#[derive(Template)]
#[template(path = "partials/_pagination_oob.html")]
pub struct PaginationOob<'a> {
    pub pagination: &'a crate::views::pagination::Pagination,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn matches_query(entry: &kanidm_proto::v1::Entry, q: &str) -> bool {
    let q_lower = q.to_lowercase();
    for field in ["name", "displayname", "spn"] {
        for v in attr_all(entry, field) {
            if v.to_lowercase().contains(&q_lower) {
                return true;
            }
        }
    }
    false
}

fn entry_to_row(entry: &kanidm_proto::v1::Entry, _kanidm_url: &str) -> OAuth2AppRow {
    let name = attr_first(entry, "name").unwrap_or_default();
    let displayname = attr_first(entry, "displayname")
        .or_else(|| attr_first(entry, "name"))
        .unwrap_or_default();

    let image_url = if attr_present(entry, "image") {
        Some(format!("/oauth2/{}/image-proxy", name))
    } else {
        None
    };

    let detail_href = format!("/oauth2/{}", name);
    let kind = detect_kind(entry);
    let actions_html = build_app_actions(&name, &displayname, kind);

    OAuth2AppRow {
        initials: initials(&displayname),
        name,
        displayname,
        image_url,
        kind,
        landing_url: attr_first(entry, "oauth2_rs_origin_landing"),
        detail_href,
        actions_html,
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn list(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    headers: HeaderMap,
    Query(params): Query<ListParams>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let entries = client
        .idm_oauth2_rs_list()
        .await
        .map_err(|e| AppError::Kanidm(format!("oauth2 list failed: {e:?}")))?;

    let total_count = entries.len();
    let q = params.q.as_deref().unwrap_or("").trim().to_string();

    if wants_json(&headers) {
        let mut items: Vec<PaletteItem> = entries
            .iter()
            .filter_map(|entry| {
                if !q.is_empty() && !matches_query(entry, &q) {
                    return None;
                }
                let name = attr_first(entry, "name").unwrap_or_default();
                if name.is_empty() {
                    return None;
                }
                let label = attr_first(entry, "displayname").unwrap_or_else(|| name.clone());
                let subtitle = attr_first(entry, "oauth2_rs_origin_landing").unwrap_or_default();
                Some(PaletteItem {
                    kind: "oauth2",
                    label,
                    subtitle,
                    href: format!("/oauth2/{name}"),
                })
            })
            .collect();
        items.sort_by_key(|a| a.label.to_lowercase());
        items.truncate(50);
        return Ok(Json(PaletteResponse { items }).into_response());
    }

    let per = params.per.unwrap_or(24).clamp(1, 200);
    let page = params.page.unwrap_or(1).max(1);

    let kanidm_url = state.config.kanidm_url.clone();

    let mut filtered: Vec<OAuth2AppRow> = entries
        .iter()
        .filter_map(|entry| {
            if !q.is_empty() && !matches_query(entry, &q) {
                return None;
            }
            Some(entry_to_row(entry, &kanidm_url))
        })
        .collect();

    filtered.sort_by_key(|a| a.displayname.to_lowercase());

    let filtered_count = filtered.len();
    let total_pages = filtered_count.div_ceil(per);
    let page = page.min(total_pages.max(1));

    let start = (page - 1) * per;
    let apps: Vec<OAuth2AppRow> = filtered.into_iter().skip(start).take(per).collect();

    let pagination = crate::views::pagination::Pagination {
        page,
        total_pages,
        filtered_count,
        per_page: per,
        base_url: "/oauth2",
        target: "#oauth2-cards",
    };

    if is_htmx {
        let cards_html = askama::Template::render(&OAuth2CardsFragment {
            apps,
            q: q.clone(),
        })
        .map_err(AppError::Template)?;
        let pagination_html = askama::Template::render(&PaginationOob {
            pagination: &pagination,
        })
        .map_err(AppError::Template)?;
        return Ok(Html(format!("{cards_html}{pagination_html}")).into_response());
    }

    let count_text = if q.is_empty() {
        let noun = if total_count == 1 { "application" } else { "applications" };
        format!("{} {}", total_count, noun)
    } else {
        let noun = if total_count == 1 { "application" } else { "applications" };
        format!("{} of {} {}", filtered_count, total_count, noun)
    };

    Ok(OAuth2ListView {
        base: BaseFields::new(&user, "oauth2"),
        apps,
        q,
        per,
        pagination,
        count_text,
    }
    .into_response())
}

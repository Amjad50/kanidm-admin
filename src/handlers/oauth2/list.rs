use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
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
    pub total_count: usize,
    pub filtered_count: usize,
    pub q: String,
    pub page: usize,
    pub per: usize,
    pub total_pages: usize,
    pub page_start: usize,
    pub page_end: usize,
}

#[derive(Template)]
#[template(path = "oauth2/_cards.html")]
pub struct OAuth2CardsFragment {
    pub apps: Vec<OAuth2AppRow>,
    pub q: String,
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

fn entry_to_row(entry: &kanidm_proto::v1::Entry, kanidm_url: &str) -> OAuth2AppRow {
    let name = attr_first(entry, "name").unwrap_or_default();
    let displayname = attr_first(entry, "displayname")
        .or_else(|| attr_first(entry, "name"))
        .unwrap_or_default();

    let image_url = if attr_present(entry, "image") {
        let base = kanidm_url.trim_end_matches('/');
        Some(format!("{}/ui/images/oauth2/{}", base, name))
    } else {
        None
    };

    let detail_href = format!("/oauth2/{}", name);

    OAuth2AppRow {
        initials: initials(&displayname),
        name,
        displayname,
        image_url,
        kind: detect_kind(entry),
        landing_url: attr_first(entry, "oauth2_rs_origin_landing"),
        detail_href,
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn list(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
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
    let per = params.per.unwrap_or(50).min(200).max(1);
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
    let page_start = if filtered_count == 0 { 0 } else { start + 1 };
    let page_end = (start + per).min(filtered_count);
    let apps: Vec<OAuth2AppRow> = filtered.into_iter().skip(start).take(per).collect();

    if is_htmx {
        let fragment = OAuth2CardsFragment {
            apps,
            q: q.clone(),
        };
        let html = askama::Template::render(&fragment).map_err(AppError::Template)?;
        return Ok(Html(html).into_response());
    }

    Ok(OAuth2ListView {
        base: BaseFields::new(&user, "oauth2"),
        apps,
        total_count,
        filtered_count,
        q,
        page,
        per,
        total_pages,
        page_start,
        page_end,
    }
    .into_response())
}

use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_all, attr_first, spn_or_uuid};
use crate::views::BaseFields;
use crate::AppState;

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct ListParams {
    pub q: Option<String>,
    pub page: Option<usize>,
    pub per: Option<usize>,
}

// ── Row data ──────────────────────────────────────────────────────────────────

pub struct GroupRow {
    pub name: String,
    pub spn_or_uuid: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub has_policy: bool,
    pub is_builtin: bool,
    pub is_dynamic: bool,
}

// ── View structs ──────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "groups/list.html")]
pub struct GroupsListView {
    pub base: BaseFields,
    pub groups: Vec<GroupRow>,
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
#[template(path = "groups/_rows.html")]
pub struct GroupRowsFragment {
    pub groups: Vec<GroupRow>,
    pub q: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn matches_query(entry: &kanidm_proto::v1::Entry, q: &str) -> bool {
    let q_lower = q.to_lowercase();
    for field in ["name", "spn", "description"] {
        for v in attr_all(entry, field) {
            if v.to_lowercase().contains(&q_lower) {
                return true;
            }
        }
    }
    false
}

fn entry_to_row(entry: &kanidm_proto::v1::Entry) -> GroupRow {
    let classes = attr_all(entry, "class");
    let is_dynamic = classes.iter().any(|c| c == "dyngroup");
    let has_policy = classes.iter().any(|c| c == "account_policy");
    let is_builtin = classes.iter().any(|c| c == "builtin");

    let member_count = if is_dynamic {
        attr_all(entry, "dynmember").len()
    } else {
        attr_all(entry, "member").len()
    };

    GroupRow {
        name: attr_first(entry, "name").unwrap_or_default(),
        spn_or_uuid: spn_or_uuid(entry),
        description: attr_first(entry, "description"),
        member_count,
        has_policy,
        is_builtin,
        is_dynamic,
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
        .idm_group_list()
        .await
        .map_err(|e| AppError::Kanidm(format!("group list failed: {e:?}")))?;

    let total_count = entries.len();
    let q = params.q.as_deref().unwrap_or("").trim().to_string();
    let per = params.per.unwrap_or(50).min(200).max(1);
    let page = params.page.unwrap_or(1).max(1);

    let mut filtered: Vec<GroupRow> = entries
        .iter()
        .filter_map(|entry| {
            if !q.is_empty() && !matches_query(entry, &q) {
                return None;
            }
            Some(entry_to_row(entry))
        })
        .collect();

    filtered.sort_by_key(|a| a.name.to_lowercase());

    let filtered_count = filtered.len();
    let total_pages = filtered_count.div_ceil(per);
    let page = page.min(total_pages.max(1));

    let start = (page - 1) * per;
    let page_start = if filtered_count == 0 { 0 } else { start + 1 };
    let page_end = (start + per).min(filtered_count);
    let groups: Vec<GroupRow> = filtered.into_iter().skip(start).take(per).collect();

    if is_htmx {
        let fragment = GroupRowsFragment { groups, q: q.clone() };
        let html = askama::Template::render(&fragment).map_err(AppError::Template)?;
        return Ok(Html(html).into_response());
    }

    Ok(GroupsListView {
        base: BaseFields::new(&user, "groups"),
        groups,
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

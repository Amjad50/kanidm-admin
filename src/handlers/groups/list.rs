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
    pub actions_html: String,
}

fn build_group_actions(spn_or_uuid: &str, name: &str, is_builtin: bool) -> String {
    use crate::views::dropdown::{render_actions_cell, DropdownItem};

    let mut items = vec![DropdownItem::link(
        "Members",
        format!("/groups/{spn_or_uuid}/members"),
    )
    .with_icon("users")];

    if !is_builtin {
        items.push(
            DropdownItem::link("Edit", format!("/groups/{spn_or_uuid}/edit")).with_icon("pencil"),
        );
        items.push(DropdownItem::Divider);
        items.push(
            DropdownItem::htmx_get("Delete", format!("/groups/{spn_or_uuid}/delete"))
                .with_icon("trash-2")
                .danger(),
        );
    }

    render_actions_cell(items, format!("Actions for {name}"))
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
    pub per: usize,
    pub pagination: crate::views::pagination::Pagination,
    pub count_text: String,
}

#[derive(Template)]
#[template(path = "groups/_rows.html")]
pub struct GroupRowsFragment {
    pub groups: Vec<GroupRow>,
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

    let id = spn_or_uuid(entry);
    let name = attr_first(entry, "name").unwrap_or_default();
    let actions_html = build_group_actions(&id, &name, is_builtin);
    GroupRow {
        name,
        spn_or_uuid: id,
        description: attr_first(entry, "description"),
        member_count,
        has_policy,
        is_builtin,
        is_dynamic,
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
        .idm_group_list()
        .await
        .map_err(|e| AppError::Kanidm(format!("group list failed: {e:?}")))?;

    let total_count = entries.len();
    let q = params.q.as_deref().unwrap_or("").trim().to_string();

    if wants_json(&headers) {
        let mut items: Vec<PaletteItem> = entries
            .iter()
            .filter_map(|entry| {
                if !q.is_empty() && !matches_query(entry, &q) {
                    return None;
                }
                let label = attr_first(entry, "name").unwrap_or_default();
                if label.is_empty() {
                    return None;
                }
                let subtitle = attr_first(entry, "description").unwrap_or_default();
                let id = spn_or_uuid(entry);
                Some(PaletteItem {
                    kind: "group",
                    label,
                    subtitle,
                    href: format!("/groups/{id}"),
                })
            })
            .collect();
        items.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
        items.truncate(50);
        return Ok(Json(PaletteResponse { items }).into_response());
    }

    let per = params.per.unwrap_or(15).min(200).max(1);
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
    let groups: Vec<GroupRow> = filtered.into_iter().skip(start).take(per).collect();

    if is_htmx {
        let pagination = crate::views::pagination::Pagination {
            page,
            total_pages,
            filtered_count,
            per_page: per,
            base_url: "/groups",
            target: "#groups-tbody",
        };
        let rows_html = askama::Template::render(&GroupRowsFragment {
            groups,
            q: q.clone(),
        })
        .map_err(AppError::Template)?;
        let pagination_html = askama::Template::render(&PaginationOob {
            pagination: &pagination,
        })
        .map_err(AppError::Template)?;
        return Ok(Html(format!("{rows_html}{pagination_html}")).into_response());
    }

    let count_text = if q.is_empty() {
        let noun = if total_count == 1 { "group" } else { "groups" };
        format!("{} {}", total_count, noun)
    } else {
        let noun = if total_count == 1 { "group" } else { "groups" };
        format!("{} of {} {}", filtered_count, total_count, noun)
    };

    Ok(GroupsListView {
        base: BaseFields::new(&user, "groups"),
        groups,
        total_count,
        filtered_count,
        q,
        per,
        pagination: crate::views::pagination::Pagination {
            page,
            total_pages,
            filtered_count,
            per_page: per,
            base_url: "/groups",
            target: "#groups-tbody",
        },
        count_text,
    }
    .into_response())
}

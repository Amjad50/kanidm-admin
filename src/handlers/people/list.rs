use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use axum_htmx::HxRequest;
use time::OffsetDateTime;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_first, spn_or_uuid};
use crate::views::{initials, BaseFields};
use crate::AppState;

use super::common::{compute_status_at, PersonStatus};
use crate::handlers::common::{wants_json, PaletteItem, PaletteResponse};

#[derive(serde::Deserialize, Default)]
pub struct ListParams {
    pub q: Option<String>,
    #[serde(default)]
    pub status: StatusFilter,
    pub page: Option<usize>,
    pub per: Option<usize>,
}

#[derive(serde::Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusFilter {
    #[default]
    All,
    Active,
    Expired,
    NotYetActive,
    NoCredentials,
}

impl StatusFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            StatusFilter::All => "all",
            StatusFilter::Active => "active",
            StatusFilter::Expired => "expired",
            StatusFilter::NotYetActive => "not_yet_active",
            StatusFilter::NoCredentials => "no_credentials",
        }
    }
}


pub struct PersonRow {
    pub initials: String,
    pub displayname: String,
    pub spn: String,
    pub mail: String,
    pub status: PersonStatus,
    pub spn_or_uuid: String,
    /// Pre-rendered actions cell HTML (kebab + dropdown, or single icon).
    pub actions_html: String,
}

fn build_person_actions(spn_or_uuid: &str, displayname: &str) -> String {
    use crate::views::dropdown::{render_actions_cell, DropdownItem};
    render_actions_cell(
        vec![
            DropdownItem::link("Edit", format!("/admin/people/{spn_or_uuid}/edit")).with_icon("pencil"),
            DropdownItem::htmx_get(
                "Generate reset link",
                format!("/admin/people/{spn_or_uuid}/credentials/reset"),
            )
            .with_icon("refresh-cw"),
            DropdownItem::Divider,
            DropdownItem::htmx_get("Delete", format!("/admin/people/{spn_or_uuid}/delete"))
                .with_icon("trash-2")
                .danger(),
        ],
        format!("Actions for {displayname}"),
    )
}

#[derive(Template, WebTemplate)]
#[template(path = "people/list.html")]
pub struct PeopleListView {
    pub base: BaseFields,
    pub people: Vec<PersonRow>,
    pub q: String,
    pub status: StatusFilter,
    pub per: usize,
    pub pagination: crate::views::pagination::Pagination,
    pub count_text: String,
}

#[derive(Template)]
#[template(path = "people/_rows.html")]
pub struct PeopleRowsFragment {
    pub people: Vec<PersonRow>,
    pub q: String,
}

#[derive(Template)]
#[template(path = "partials/_pagination_oob.html")]
pub struct PaginationOob<'a> {
    pub pagination: &'a crate::views::pagination::Pagination,
}


fn matches_query(entry: &kanidm_proto::v1::Entry, q: &str) -> bool {
    let q_lower = q.to_lowercase();
    let fields = ["name", "spn", "displayname", "mail"];
    for field in fields {
        if let Some(values) = entry.attrs.get(field) {
            for v in values {
                if v.to_lowercase().contains(&q_lower) {
                    return true;
                }
            }
        }
    }
    false
}

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
        .idm_person_account_list()
        .await
        .map_err(|e| AppError::Kanidm(format!("person list failed: {e:?}")))?;

    let total_count = entries.len();

    let q = params.q.as_deref().unwrap_or("").trim().to_string();

    // Checked before HTMX so explicit `Accept: application/json` wins.
    // Sort by displayname, cap at 50.
    if wants_json(&headers) {
        let mut items: Vec<PaletteItem> = entries
            .iter()
            .filter_map(|entry| {
                if !q.is_empty() && !matches_query(entry, &q) {
                    return None;
                }
                let label = attr_first(entry, "displayname")
                    .or_else(|| attr_first(entry, "name"))
                    .unwrap_or_default();
                if label.is_empty() {
                    return None;
                }
                let subtitle = attr_first(entry, "spn").unwrap_or_default();
                let id = spn_or_uuid(entry);
                Some(PaletteItem {
                    kind: "person",
                    label,
                    subtitle,
                    href: format!("/admin/people/{id}"),
                })
            })
            .collect();
        items.sort_by_key(|a| a.label.to_lowercase());
        items.truncate(50);
        return Ok(Json(PaletteResponse { items }).into_response());
    }

    let status_filter = params.status;
    let per = params.per.unwrap_or(15).clamp(1, 200);
    let page = params.page.unwrap_or(1).max(1);

    let now = OffsetDateTime::now_utc();
    let mut filtered: Vec<PersonRow> = entries
        .iter()
        .filter_map(|entry| {
            if !q.is_empty() && !matches_query(entry, &q) {
                return None;
            }
            let status = compute_status_at(entry, now);
            let matches_status = match status_filter {
                StatusFilter::All => true,
                StatusFilter::Active => status == PersonStatus::Active,
                StatusFilter::Expired => status == PersonStatus::Expired,
                StatusFilter::NotYetActive => status == PersonStatus::NotYetActive,
                StatusFilter::NoCredentials => !entry.attrs.contains_key("primary_credential"),
            };
            if !matches_status {
                return None;
            }
            let displayname = attr_first(entry, "displayname")
                .or_else(|| attr_first(entry, "name"))
                .unwrap_or_default();
            let spn = attr_first(entry, "spn").unwrap_or_default();
            let mail = attr_first(entry, "mail").unwrap_or_default();
            let id = spn_or_uuid(entry);
            let actions_html = build_person_actions(&id, &displayname);
            Some(PersonRow {
                initials: initials(&displayname),
                displayname,
                spn,
                mail,
                status,
                spn_or_uuid: id,
                actions_html,
            })
        })
        .collect();

    filtered.sort_by(|a, b| {
        a.displayname
            .to_lowercase()
            .cmp(&b.displayname.to_lowercase())
    });

    let filtered_count = filtered.len();
    let total_pages = filtered_count.div_ceil(per);
    let page = page.min(total_pages.max(1));

    let start = (page - 1) * per;
    let people: Vec<PersonRow> = filtered.into_iter().skip(start).take(per).collect();

    if is_htmx {
        let pagination = crate::views::pagination::Pagination {
            page,
            total_pages,
            filtered_count,
            per_page: per,
            base_url: "/admin/people",
            target: "#people-tbody",
        };
        let rows_html = askama::Template::render(&PeopleRowsFragment {
            people,
            q: q.clone(),
        })
        .map_err(AppError::Template)?;
        let pagination_html = askama::Template::render(&PaginationOob {
            pagination: &pagination,
        })
        .map_err(AppError::Template)?;
        return Ok(Html(format!("{rows_html}{pagination_html}")).into_response());
    }

    let count_text = if q.is_empty() && status_filter == StatusFilter::All {
        let noun = if total_count == 1 { "person" } else { "people" };
        format!("{} {}", total_count, noun)
    } else {
        let noun = if total_count == 1 { "person" } else { "people" };
        format!("{} of {} {}", filtered_count, total_count, noun)
    };

    let view = PeopleListView {
        base: BaseFields::new(&user, "people"),
        people,
        q,
        status: status_filter,
        per,
        pagination: crate::views::pagination::Pagination {
            page,
            total_pages,
            filtered_count,
            per_page: per,
            base_url: "/admin/people",
            target: "#people-tbody",
        },
        count_text,
    };

    Ok(view.into_response())
}


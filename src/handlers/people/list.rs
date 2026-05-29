use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;
use time::OffsetDateTime;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_first, spn_or_uuid};
use crate::views::{initials, BaseFields};
use crate::AppState;

use super::common::{compute_status_at, PersonStatus};

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
}

#[derive(Template, WebTemplate)]
#[template(path = "people/list.html")]
pub struct PeopleListView {
    pub base: BaseFields,
    pub people: Vec<PersonRow>,
    pub total_count: usize,
    pub filtered_count: usize,
    pub q: String,
    pub status: StatusFilter,
    pub page: usize,
    pub per: usize,
    pub total_pages: usize,
    pub page_start: usize,
    pub page_end: usize,
}

#[derive(Template)]
#[template(path = "people/_rows.html")]
pub struct PeopleRowsFragment {
    pub people: Vec<PersonRow>,
    pub q: String,
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
    let status_filter = params.status;
    let per = params.per.unwrap_or(50).min(200).max(1);
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
                StatusFilter::NoCredentials => entry.attrs.get("primary_credential").is_none(),
            };
            if !matches_status {
                return None;
            }
            let displayname = attr_first(entry, "displayname")
                .or_else(|| attr_first(entry, "name"))
                .unwrap_or_default();
            let spn = attr_first(entry, "spn").unwrap_or_default();
            let mail = attr_first(entry, "mail").unwrap_or_default();
            Some(PersonRow {
                initials: initials(&displayname),
                displayname,
                spn,
                mail,
                status,
                spn_or_uuid: spn_or_uuid(entry),
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
    let page_start = if filtered_count == 0 { 0 } else { start + 1 };
    let page_end = (start + per).min(filtered_count);
    let people: Vec<PersonRow> = filtered.into_iter().skip(start).take(per).collect();

    if is_htmx {
        let fragment = PeopleRowsFragment {
            people,
            q: q.clone(),
        };
        let html = askama::Template::render(&fragment)
            .map_err(AppError::Template)?;
        return Ok(Html(html).into_response());
    }

    let view = PeopleListView {
        base: BaseFields::new(&user, "people"),
        people,
        total_count,
        filtered_count,
        q,
        status: status_filter,
        page,
        per,
        total_pages,
        page_start,
        page_end,
    };

    Ok(view.into_response())
}


use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_all, attr_first};
use crate::views::BaseFields;
use crate::AppState;

use super::common::{compute_header, fetch_group, spn_initials, GroupHeader};
use super::members::MembersData;
use super::policy::PolicyData;

// ── Tab definitions ───────────────────────────────────────────────────────────

pub struct TabDef {
    pub slug: &'static str,
    pub label: &'static str,
}

pub const TABS: &[TabDef] = &[
    TabDef { slug: "overview", label: "Overview" },
    TabDef { slug: "members",  label: "Members"  },
    TabDef { slug: "policy",   label: "Account policy" },
];

// ── Overview tab data ─────────────────────────────────────────────────────────

pub struct MemberChip {
    pub initials: String,
    pub name: String,
    pub spn_or_id: String,
}

pub struct OverviewData {
    pub uuid: String,
    pub name: String,
    pub spn: String,
    pub description: Option<String>,
    pub mails: Vec<String>,
    pub entry_managed_by: Option<String>,
    pub members_preview: Vec<MemberChip>,
    pub member_count: usize,
    pub members_overflow: usize,
    /// Account policy summary fields (only populated when has_policy is true)
    pub policy_summary: Vec<(String, String)>,
}

// ── Tab content enum ──────────────────────────────────────────────────────────

pub enum TabContent {
    Overview(OverviewData),
    Members(MembersData),
    Policy(PolicyData),
}

// ── Full-page view ────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "groups/detail.html")]
pub struct DetailView {
    pub base: BaseFields,
    pub group: GroupHeader,
    pub active_tab: &'static str,
    pub tabs: &'static [TabDef],
    pub tab_content: TabContent,
    pub oob: bool,
}

impl IntoResponse for DetailView {
    fn into_response(self) -> Response {
        match askama::Template::render(&self) {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::Template(e).into_response(),
        }
    }
}

// ── HTMX fragments ────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "groups/_tab_content_fragment.html")]
pub struct TabContentFragment<'a> {
    pub tab_content: &'a TabContent,
    pub group: &'a GroupHeader,
}

#[derive(Template)]
#[template(path = "groups/_tabs_nav.html")]
pub struct TabsNavFragment<'a> {
    pub group: &'a GroupHeader,
    pub active_tab: &'static str,
    pub tabs: &'static [TabDef],
    pub oob: bool,
}

// ── Shared render helper ──────────────────────────────────────────────────────

pub(super) fn render_detail(
    is_htmx: bool,
    user: AdminUser,
    group: GroupHeader,
    active_tab: &'static str,
    tab_content: TabContent,
) -> AppResult<Response> {
    if is_htmx {
        let content_html = askama::Template::render(&TabContentFragment {
            tab_content: &tab_content,
            group: &group,
        })
        .map_err(AppError::Template)?;

        let nav_html = askama::Template::render(&TabsNavFragment {
            group: &group,
            active_tab,
            tabs: TABS,
            oob: true,
        })
        .map_err(AppError::Template)?;

        return Ok(Html(format!("{content_html}{nav_html}")).into_response());
    }

    Ok(DetailView {
        base: BaseFields::new(&user, "groups"),
        group,
        active_tab,
        tabs: TABS,
        tab_content,
        oob: false,
    }
    .into_response())
}

// ── Data builders ─────────────────────────────────────────────────────────────

fn build_overview(entry: &kanidm_proto::v1::Entry) -> OverviewData {
    let uuid = attr_first(entry, "uuid").unwrap_or_default();
    let name = attr_first(entry, "name").unwrap_or_default();
    let spn = attr_first(entry, "spn").unwrap_or_default();
    let description = attr_first(entry, "description");
    let mails = attr_all(entry, "mail");
    let entry_managed_by = attr_first(entry, "entry_managed_by");

    let classes = attr_all(entry, "class");
    let is_dynamic = classes.iter().any(|c| c == "dyngroup");
    let has_policy = classes.iter().any(|c| c == "account_policy");

    let all_members = if is_dynamic {
        attr_all(entry, "dynmember")
    } else {
        attr_all(entry, "member")
    };
    let member_count = all_members.len();
    let preview_max = 6;
    let members_overflow = member_count.saturating_sub(preview_max);
    let members_preview: Vec<MemberChip> = all_members
        .into_iter()
        .take(preview_max)
        .map(|spn| {
            let name_part = spn.split('@').next().unwrap_or(&spn).to_string();
            MemberChip {
                initials: spn_initials(&spn),
                name: name_part,
                spn_or_id: spn,
            }
        })
        .collect();

    // Build a concise policy summary for the overview card
    let mut policy_summary = Vec::new();
    if has_policy {
        if let Some(v) = attr_first(entry, "credential_type_minimum") {
            policy_summary.push(("Credential type minimum".to_string(), v));
        }
        if let Some(v) = attr_first(entry, "auth_password_minimum_length") {
            policy_summary.push(("Password minimum length".to_string(), format!("{v} characters")));
        }
        if let Some(v) = attr_first(entry, "authsession_expiry") {
            let secs: u64 = v.parse().unwrap_or(0);
            let display = format_seconds(secs);
            policy_summary.push(("Session expiry".to_string(), format!("{v} s ({display})")));
        }
        if let Some(v) = attr_first(entry, "privilege_expiry") {
            let secs: u64 = v.parse().unwrap_or(0);
            let display = format_seconds(secs);
            policy_summary.push(("Privilege expiry".to_string(), format!("{v} s ({display})")));
        }
    }

    OverviewData {
        uuid,
        name,
        spn,
        description,
        mails,
        entry_managed_by,
        members_preview,
        member_count,
        members_overflow,
        policy_summary,
    }
}

pub(super) fn format_seconds(secs: u64) -> String {
    if secs == 0 {
        return "0 s".to_string();
    }
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let remainder = secs % 60;
    if hours > 0 && minutes == 0 && remainder == 0 {
        return format!("{hours} h");
    }
    if hours > 0 && remainder == 0 {
        return format!("{hours} h {minutes} min");
    }
    if hours > 0 {
        return format!("{hours} h {minutes} min {remainder} s");
    }
    if minutes > 0 && remainder == 0 {
        return format!("{minutes} min");
    }
    if minutes > 0 {
        return format!("{minutes} min {remainder} s");
    }
    format!("{secs} s")
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /groups/{id} → 308 to /groups/{id}/overview
pub async fn redirect_to_overview(Path(id): Path<String>) -> Redirect {
    Redirect::permanent(&format!("/groups/{id}/overview"))
}

/// GET /groups/{id}/overview
pub async fn overview(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let tab_content = TabContent::Overview(build_overview(&entry));
    render_detail(is_htmx, user, group, "overview", tab_content)
}

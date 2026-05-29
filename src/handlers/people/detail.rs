use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_htmx::HxRequest;
use time::OffsetDateTime;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_all, attr_first, attr_present, spn_or_uuid};
use crate::views::{initials, BaseFields};
use crate::AppState;

use super::common::{compute_status_at, parse_kanidm_datetime, summarize_credentials, CredentialSummary};
use super::credentials::CredentialsData;
use super::groups_tab::GroupsTabData;
use super::radius::RadiusData;
use super::sessions::SessionsData;
use super::ssh::SshData;
use super::validity::ValidityData;

pub struct TabDef {
    pub slug: &'static str,
    pub label: &'static str,
}

pub const TABS: &[TabDef] = &[
    TabDef { slug: "overview",     label: "Overview"     },
    TabDef { slug: "credentials",  label: "Credentials"  },
    TabDef { slug: "ssh",          label: "SSH Keys"      },
    TabDef { slug: "radius",       label: "RADIUS"        },
    TabDef { slug: "sessions",     label: "Sessions"      },
    TabDef { slug: "groups",       label: "Groups"        },
    TabDef { slug: "validity",     label: "Validity"      },
];

/// Header info shown on every detail tab.
pub struct PersonHeader {
    pub initials: String,
    pub displayname: String,
    pub spn: String,
    pub spn_or_uuid: String,
    pub primary_mail: String,
    pub status_label: &'static str,
    pub status_badge_classes: &'static str,
    pub status_dot_classes: &'static str,
}

pub struct GroupChip {
    pub name: String,
    pub spn_or_id: String,
}

pub struct OverviewData {
    pub uuid: String,
    pub name: String,
    pub displayname: String,
    pub legalname: Option<String>,
    pub mails: Vec<String>,
    pub groups: Vec<GroupChip>,
    pub direct_group_count: usize,
    pub valid_from: Option<String>,
    pub expire_at: Option<String>,
    pub credential_summary: CredentialSummary,
}

pub enum TabContent {
    Overview(OverviewData),
    Credentials(CredentialsData),
    Ssh(SshData),
    Radius(RadiusData),
    Sessions(SessionsData),
    Groups(GroupsTabData),
    Validity(ValidityData),
}

/// Full-page detail view (non-HTMX requests).
/// `oob` is always `false` here; it only exists so `_tabs_nav.html` can be
/// `{% include %}`d from both this template and the standalone OOB fragment.
#[derive(Template)]
#[template(path = "people/detail.html")]
pub struct DetailView {
    pub base: BaseFields,
    pub person: PersonHeader,
    pub active_tab: &'static str,
    pub tabs: &'static [TabDef],
    pub tab_content: TabContent,
    /// Always `false` in the full-page render; `_tabs_nav.html` reads this.
    pub oob: bool,
}

impl axum::response::IntoResponse for DetailView {
    fn into_response(self) -> Response {
        match askama::Template::render(&self) {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::Template(e).into_response(),
        }
    }
}

/// Partial fragment: just the tab body. Used for HTMX content swaps.
/// Carries `person` because `_tab_overview.html` needs it for link targets.
#[derive(Template)]
#[template(path = "people/_tab_content_fragment.html")]
pub struct TabContentFragment<'a> {
    pub tab_content: &'a TabContent,
    pub person: &'a PersonHeader,
}

/// Standalone nav — used as an OOB swap response alongside tab content.
#[derive(Template)]
#[template(path = "people/_tabs_nav.html")]
pub struct TabsNavFragment<'a> {
    pub person: &'a PersonHeader,
    pub active_tab: &'static str,
    pub tabs: &'static [TabDef],
    pub oob: bool,
}

fn format_validity_display(val: Option<String>) -> Option<String> {
    let s = val?;
    let dt = parse_kanidm_datetime(&s)?;
    let now = OffsetDateTime::now_utc();
    if dt <= now {
        Some(crate::views::format_relative_past(dt))
    } else {
        Some(crate::views::format_relative_future(dt))
    }
}

pub(super) fn compute_header(entry: &kanidm_proto::v1::Entry) -> PersonHeader {
    let now = OffsetDateTime::now_utc();
    let status = compute_status_at(entry, now);

    let displayname = attr_first(entry, "displayname")
        .or_else(|| attr_first(entry, "name"))
        .unwrap_or_default();
    let spn = attr_first(entry, "spn").unwrap_or_default();
    let primary_mail = attr_first(entry, "mail").unwrap_or_default();

    PersonHeader {
        initials: initials(&displayname),
        displayname,
        spn,
        spn_or_uuid: spn_or_uuid(entry),
        primary_mail,
        status_label: status.label(),
        status_badge_classes: status.badge_classes(),
        status_dot_classes: status.dot_classes(),
    }
}

fn build_overview(entry: &kanidm_proto::v1::Entry) -> OverviewData {
    let uuid = attr_first(entry, "uuid").unwrap_or_default();
    let name = attr_first(entry, "name").unwrap_or_default();
    let displayname = attr_first(entry, "displayname")
        .unwrap_or_else(|| name.clone());
    let legalname = attr_first(entry, "legalname");
    let mails = attr_all(entry, "mail");

    // Groups: prefer directmemberof (direct only), fall back to memberof (transitive)
    let group_spns = if attr_present(entry, "directmemberof") {
        attr_all(entry, "directmemberof")
    } else {
        attr_all(entry, "memberof")
    };
    let direct_group_count = group_spns.len();
    // Show only the first 5 groups inline; the dedicated Groups tab has the full list.
    let groups: Vec<GroupChip> = group_spns
        .into_iter()
        .take(5)
        .map(|spn| {
            let name = spn.split('@').next().unwrap_or(&spn).to_string();
            GroupChip { name, spn_or_id: spn }
        })
        .collect();

    let valid_from = format_validity_display(attr_first(entry, "account_valid_from"));
    let expire_at  = format_validity_display(attr_first(entry, "account_expire"));

    // Credential summary — derived from entry attrs only; no extra API call on the Overview tab.
    let credential_summary = summarize_credentials(entry, None);

    OverviewData {
        uuid,
        name,
        displayname,
        legalname,
        mails,
        groups,
        direct_group_count,
        valid_from,
        expire_at,
        credential_summary,
    }
}

/// GET /people/{id}  →  308 to /people/{id}/overview
pub async fn redirect_to_overview(Path(id): Path<String>) -> Redirect {
    Redirect::permanent(&format!("/people/{id}/overview"))
}

/// GET /people/{id}/overview
pub async fn overview(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let tab_content = TabContent::Overview(build_overview(&entry));
    render_detail(is_htmx, user, person, "overview", tab_content)
}


pub(super) async fn fetch_person(
    state: &AppState,
    user: &AdminUser,
    id: &str,
) -> AppResult<kanidm_proto::v1::Entry> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    client
        .idm_person_account_get(id)
        .await
        .map_err(|e| AppError::Kanidm(format!("person get failed: {e:?}")))?
        .ok_or(AppError::NotFound)
}

pub(super) fn render_detail(
    is_htmx: bool,
    user: AdminUser,
    person: PersonHeader,
    active_tab: &'static str,
    tab_content: TabContent,
) -> AppResult<Response> {
    if is_htmx {
        let content_html = askama::Template::render(&TabContentFragment {
            tab_content: &tab_content,
            person: &person,
        })
        .map_err(AppError::Template)?;

        let nav_html = askama::Template::render(&TabsNavFragment {
            person: &person,
            active_tab,
            tabs: TABS,
            oob: true,
        })
        .map_err(AppError::Template)?;

        return Ok(Html(format!("{content_html}{nav_html}")).into_response());
    }

    Ok(DetailView {
        base: BaseFields::new(&user, "people"),
        person,
        active_tab,
        tabs: TABS,
        tab_content,
        oob: false,
    }
    .into_response())
}

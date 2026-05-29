use askama::Template;
use axum::extract::Path;
use axum::response::{Html, IntoResponse, Redirect, Response};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::{attr_first, attr_present};
use crate::views::{initials, BaseFields};
use crate::AppState;

use super::common::{detect_kind, OAuth2Kind};
use super::general::GeneralData;

// ── Tab definitions ───────────────────────────────────────────────────────────

pub struct TabDef {
    pub slug: &'static str,
    pub label: &'static str,
}

pub const TABS: &[TabDef] = &[
    TabDef { slug: "overview",   label: "Overview"   },
    TabDef { slug: "general",    label: "General"    },
    TabDef { slug: "scope-maps", label: "Scope maps" },
    TabDef { slug: "claim-maps", label: "Claim maps" },
    TabDef { slug: "crypto",     label: "Crypto"     },
    TabDef { slug: "image",      label: "Image"      },
    TabDef { slug: "advanced",   label: "Advanced"   },
];

// ── Header ────────────────────────────────────────────────────────────────────

/// Data shown above every tab (avatar, name, kind badge, copy chips).
pub struct OAuth2Header {
    pub name: String,
    pub displayname: String,
    pub kind: OAuth2Kind,
    pub kind_label: &'static str,
    pub kind_badge_classes: &'static str,
    pub initials: String,
    pub image_url: Option<String>,
    pub uuid: String,
    pub spn: String,
}

// ── Placeholder data (used by not-yet-implemented tabs) ───────────────────────

pub struct PlaceholderTabData {
    // Read by templates/oauth2/_tab_content_fragment.html and detail.html match arms.
    #[allow(dead_code)]
    pub tab_name: &'static str,
}

// ── Tab content enum ──────────────────────────────────────────────────────────

pub enum TabContent {
    Overview(super::overview::OverviewData),
    General(GeneralData),
    // Reserved for not-yet-implemented tabs; templates have match arms ready.
    #[allow(dead_code)]
    Placeholder(PlaceholderTabData),
    ScopeMaps(super::scope_maps::ScopeMapsData),
    ClaimMaps(super::claim_maps::ClaimMapsData),
    Crypto(super::crypto::CryptoData),
    Image(super::image::ImageData),
    Advanced(super::advanced::AdvancedData),
}

// ── Full-page view ────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "oauth2/detail.html")]
pub struct DetailView {
    pub base: BaseFields,
    pub header: OAuth2Header,
    pub active_tab: &'static str,
    pub tabs: &'static [TabDef],
    pub tab_content: TabContent,
    /// Always `false` in the full-page render; `_tabs_nav.html` reads this.
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
#[template(path = "oauth2/_tab_content_fragment.html")]
pub struct TabContentFragment<'a> {
    pub tab_content: &'a TabContent,
}

#[derive(Template)]
#[template(path = "oauth2/_tabs_nav.html")]
pub struct TabsNavFragment<'a> {
    pub header: &'a OAuth2Header,
    pub active_tab: &'static str,
    pub tabs: &'static [TabDef],
    pub oob: bool,
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Compute the shared header data from a raw entry.
pub(super) fn compute_header(_state: &AppState, entry: &kanidm_proto::v1::Entry) -> OAuth2Header {
    let name = attr_first(entry, "name").unwrap_or_default();
    let displayname = attr_first(entry, "displayname")
        .or_else(|| attr_first(entry, "name"))
        .unwrap_or_default();
    let uuid = attr_first(entry, "uuid").unwrap_or_default();
    let spn = attr_first(entry, "spn").unwrap_or_default();

    let image_url = if attr_present(entry, "image") {
        Some(format!("/admin/oauth2/{}/image-proxy", name))
    } else {
        None
    };

    let kind = detect_kind(entry);
    let kind_label = kind.full_label();
    let kind_badge_classes = kind.badge_classes();

    OAuth2Header {
        initials: initials(&displayname),
        name,
        displayname,
        kind,
        kind_label,
        kind_badge_classes,
        image_url,
        uuid,
        spn,
    }
}

/// Fetch an OAuth2 entry by name/id from Kanidm.
pub(super) async fn fetch_oauth2_entry(
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
        .idm_oauth2_rs_get(id)
        .await
        .map_err(|e| AppError::Kanidm(format!("oauth2 get failed: {e:?}")))?
        .ok_or(AppError::NotFound)
}

/// Render the full-page or HTMX fragment for a detail tab.
pub(super) fn render_detail(
    is_htmx: bool,
    user: AdminUser,
    header: OAuth2Header,
    active_tab: &'static str,
    tab_content: TabContent,
) -> AppResult<Response> {
    if is_htmx {
        let content_html = askama::Template::render(&TabContentFragment {
            tab_content: &tab_content,
        })
        .map_err(AppError::Template)?;

        let nav_html = askama::Template::render(&TabsNavFragment {
            header: &header,
            active_tab,
            tabs: TABS,
            oob: true,
        })
        .map_err(AppError::Template)?;

        return Ok(Html(format!("{content_html}{nav_html}")).into_response());
    }

    Ok(DetailView {
        base: BaseFields::new(&user, "oauth2"),
        header,
        active_tab,
        tabs: TABS,
        tab_content,
        oob: false,
    }
    .into_response())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id} → 308 to /oauth2/{id}/overview
pub async fn redirect_to_overview(Path(id): Path<String>) -> Redirect {
    Redirect::permanent(&format!("/admin/oauth2/{id}/overview"))
}

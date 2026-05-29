use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::views::BaseFields;
use crate::AppState;

#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
pub struct DashboardView {
    pub base: BaseFields,
    pub person_count: Option<usize>,
    pub group_count: Option<usize>,
    pub oauth2_count: Option<usize>,
    pub domain_name: Option<String>,
    pub domain_display_name: Option<String>,
}

pub async fn dashboard(
    State(state): State<AppState>,
    user: AdminUser,
) -> AppResult<DashboardView> {
    // Build a kanidm client carrying the admin's token; all queries below run
    // as that user, with their permissions and audit trail.
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| crate::error::AppError::Kanidm(e.to_string()))?;

    // Counts: fire in parallel for speed. We tolerate individual failures so a
    // dashboard with one failed card still renders.
    let (persons, groups, oauth2s, domain) = tokio::join!(
        client.idm_person_account_list(),
        client.idm_group_list(),
        client.idm_oauth2_rs_list(),
        client.idm_domain_get(),
    );

    let person_count = persons.ok().map(|v| v.len());
    let group_count = groups.ok().map(|v| v.len());
    let oauth2_count = oauth2s.ok().map(|v| v.len());

    let (domain_name, domain_display_name) = match domain {
        Ok(entry) => (
            attr_first(&entry, "name"),
            attr_first(&entry, "domain_display_name").or_else(|| attr_first(&entry, "displayname")),
        ),
        Err(_) => (None, None),
    };

    Ok(DashboardView {
        base: BaseFields::new(&user, "dashboard"),
        person_count,
        group_count,
        oauth2_count,
        domain_name,
        domain_display_name,
    })
}

/// Extract the first value of an attr from a kanidm `Entry` (the flat
/// `BTreeMap<String, Vec<String>>` shape).
fn attr_first(entry: &kanidm_proto::v1::Entry, name: &str) -> Option<String> {
    entry.attrs.get(name).and_then(|v| v.first().cloned())
}

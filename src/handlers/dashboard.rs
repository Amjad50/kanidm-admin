use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::kanidm::entry::attr_first;
use crate::views::{format_relative_future, format_relative_past, format_relative_remaining, BaseFields};
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
    pub domain_level: Option<String>,
    pub ldap_basedn: Option<String>,
    pub domain_uuid: Option<String>,
    pub signed_in_relative: Option<String>,
    pub session_expires_relative: Option<String>,
    pub privileged_remaining: Option<String>,
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

    let (domain_name, domain_display_name, domain_level, ldap_basedn, domain_uuid) = match domain {
        Ok(entry) => (
            attr_first(&entry, "domain_name").or_else(|| attr_first(&entry, "name")),
            attr_first(&entry, "domain_display_name").or_else(|| attr_first(&entry, "displayname")),
            attr_first(&entry, "domain_level"),
            attr_first(&entry, "domain_ldap_basedn").or_else(|| attr_first(&entry, "ldap_basedn")),
            attr_first(&entry, "domain_uuid").or_else(|| attr_first(&entry, "uuid")),
        ),
        Err(e) => {
            tracing::warn!(error = ?e, "idm_domain_get failed; instance card will show placeholders");
            (None, None, None, None, None)
        }
    };

    let signed_in_relative = user.signed_in_at.map(format_relative_past);
    let session_expires_relative = user.session_expires_at.map(format_relative_future);
    let privileged_remaining = if user.privileged {
        user.privileged_until.map(format_relative_remaining)
    } else {
        None
    };

    Ok(DashboardView {
        base: BaseFields::new(&user, "dashboard"),
        person_count,
        group_count,
        oauth2_count,
        domain_name,
        domain_display_name,
        domain_level,
        ldap_basedn,
        domain_uuid,
        signed_in_relative,
        session_expires_relative,
        privileged_remaining,
    })
}


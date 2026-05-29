use std::collections::HashMap;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::Form;
use axum_htmx::HxRequest;
use time::OffsetDateTime;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::people::common::{compute_status_at, PersonStatus};
use crate::kanidm::entry::{attr_all, attr_first};
use crate::views::dropdown::{render_actions_cell, DropdownItem};
use crate::views::initials;
use crate::views::partials::{DeleteFooter, DestructiveConfirm, IdentityRow, Modal};
use crate::AppState;

use super::common::{compute_header, fetch_group, friendly_error, spn_initials, GroupHeader};
use super::detail::{render_detail, TabContent};

// ── Member row data ───────────────────────────────────────────────────────────

pub struct MemberRow {
    pub initials: String,
    pub displayname: String,
    pub spn: String,
    pub mail: String,
    pub status: Option<PersonStatus>,
    pub actions_html: String,
}

/// All data needed to render the Members tab.
pub struct MembersData {
    pub members: Vec<MemberRow>,
    pub is_dynamic: bool,
    pub people_spns: Vec<String>,
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "groups/_tab_members.html")]
pub struct MembersTabFragment<'a> {
    pub data: &'a MembersData,
    pub group: &'a GroupHeader,
}

#[derive(Template)]
#[template(path = "groups/_members_list.html")]
pub struct MembersListFragment<'a> {
    pub data: &'a MembersData,
    pub group: &'a GroupHeader,
}

// ── Form structs ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AddMemberForm {
    pub member: String,
}

// ── Data helpers ──────────────────────────────────────────────────────────────

/// URL-encode a member id for safe use in route paths.
/// Kanidm SPNs contain `@` which must be percent-encoded.
pub fn encode_member_id(id: &str) -> String {
    id.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            _ => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes
                    .bytes()
                    .flat_map(|b| format!("%{b:02X}").chars().collect::<Vec<_>>())
                    .collect()
            }
        })
        .collect()
}

fn build_member_row(
    group_id: &str,
    spn: &str,
    person_entry: Option<&kanidm_proto::v1::Entry>,
    now: OffsetDateTime,
) -> MemberRow {
    let displayname = person_entry
        .and_then(|e| attr_first(e, "displayname"))
        .unwrap_or_else(|| spn.split('@').next().unwrap_or(spn).to_string());
    let mail = person_entry
        .and_then(|e| attr_first(e, "mail"))
        .unwrap_or_default();
    let status = person_entry.map(|e| compute_status_at(e, now));
    let initials_str = if !displayname.is_empty() {
        initials(&displayname)
    } else {
        spn_initials(spn)
    };
    let encoded_id = encode_member_id(spn);

    let actions_html = render_actions_cell(
        vec![DropdownItem::htmx_post(
            "Remove from group",
            format!("/groups/{group_id}/members/{encoded_id}/remove"),
        )
        .with_icon("x")
        .with_target("#members-table-body")
        .with_swap("innerHTML")
        .with_confirm(format!(
            "Remove {} from this group?",
            if !displayname.is_empty() { &displayname } else { spn }
        ))
        .danger()],
        format!(
            "Remove {} from group",
            if !displayname.is_empty() { &displayname } else { spn }
        ),
    );

    MemberRow {
        initials: initials_str,
        displayname,
        spn: spn.to_string(),
        mail,
        status,
        actions_html,
    }
}

async fn build_members_data(
    state: &AppState,
    user: &AdminUser,
    group_id: &str,
    entry: &kanidm_proto::v1::Entry,
) -> MembersData {
    let classes = attr_all(entry, "class");
    let is_dynamic = classes.iter().any(|c| c == "dyngroup");

    let member_spns = if is_dynamic {
        attr_all(entry, "dynmember")
    } else {
        attr_all(entry, "member")
    };

    // Single people list fetch to index by SPN, used for both row enrichment
    // and the add-member datalist typeahead.
    let people: Vec<kanidm_proto::v1::Entry> = match state.kanidm.for_token(&user.token).await {
        Ok(client) => client.idm_person_account_list().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let person_by_spn: HashMap<String, &kanidm_proto::v1::Entry> = people
        .iter()
        .filter_map(|e| attr_first(e, "spn").map(|spn| (spn, e)))
        .collect();
    let people_spns: Vec<String> = people
        .iter()
        .filter_map(|e| attr_first(e, "spn"))
        .collect();

    let now = OffsetDateTime::now_utc();
    let members: Vec<MemberRow> = member_spns
        .iter()
        .map(|spn| build_member_row(group_id, spn, person_by_spn.get(spn).copied(), now))
        .collect();

    MembersData { members, is_dynamic, people_spns }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /groups/{id}/members
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let data = build_members_data(&state, &user, &id, &entry).await;
    let tab_content = TabContent::Members(data);
    render_detail(is_htmx, user, group, "members", tab_content)
}

/// POST /groups/{id}/members/add
pub async fn add(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<AddMemberForm>,
) -> AppResult<Response> {
    let member = form.member.trim().to_string();
    if member.is_empty() {
        return Ok(Html(String::new()).into_response());
    }

    let members: Vec<&str> = member.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);

    if group.is_dynamic {
        let data = build_members_data(&state, &user, &id, &entry).await;
        let rows_html = askama::Template::render(&MembersListFragment { data: &data, group: &group })
            .map_err(AppError::Template)?;
        let error_oob = render_members_error_oob(Some(
            "Dynamic group membership is computed automatically and cannot be edited.".to_string(),
        ));
        return Ok(Html(format!("{rows_html}{error_oob}")).into_response());
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let add_error = if let Err(e) = client.idm_group_add_members(&id, &members).await {
        tracing::warn!(group = %id, error = ?e, "failed to add members");
        Some(friendly_error("add members", &e))
    } else {
        None
    };

    // Re-fetch entry and return updated members list fragment
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let data = build_members_data(&state, &user, &id, &entry).await;

    let rows_html = askama::Template::render(&MembersListFragment {
        data: &data,
        group: &group,
    })
    .map_err(AppError::Template)?;

    let error_oob = render_members_error_oob(add_error);
    Ok(Html(format!("{rows_html}{error_oob}")).into_response())
}

/// POST /groups/{id}/members/{mid}/remove
pub async fn remove(
    State(state): State<AppState>,
    Path((id, mid)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);

    if group.is_dynamic {
        let data = build_members_data(&state, &user, &id, &entry).await;
        let rows_html = askama::Template::render(&MembersListFragment { data: &data, group: &group })
            .map_err(AppError::Template)?;
        let error_oob = render_members_error_oob(Some(
            "Dynamic group membership is computed automatically and cannot be edited.".to_string(),
        ));
        return Ok(Html(format!("{rows_html}{error_oob}")).into_response());
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let remove_error = if let Err(e) = client.idm_group_remove_members(&id, &[mid.as_str()]).await {
        tracing::warn!(group = %id, member = %mid, error = ?e, "failed to remove member");
        Some(friendly_error("remove member", &e))
    } else {
        None
    };

    // Re-fetch and return updated members list
    let entry = fetch_group(&state, &user, &id).await?;
    let group = compute_header(&entry);
    let data = build_members_data(&state, &user, &id, &entry).await;

    let rows_html = askama::Template::render(&MembersListFragment {
        data: &data,
        group: &group,
    })
    .map_err(AppError::Template)?;

    let error_oob = render_members_error_oob(remove_error);
    Ok(Html(format!("{rows_html}{error_oob}")).into_response())
}

/// Returns an HTMX OOB swap fragment that updates `#members-error`.
/// When `msg` is None the div is cleared; when Some it shows the error banner.
fn render_members_error_oob(msg: Option<String>) -> String {
    match msg {
        None => r#"<div id="members-error" hx-swap-oob="innerHTML"></div>"#.to_string(),
        Some(text) => format!(
            r#"<div id="members-error" hx-swap-oob="innerHTML"><div class="text-danger text-sm bg-danger-soft border border-danger rounded px-3 py-2">{text}</div></div>"#,
        ),
    }
}


/// POST /groups/{id}/members/purge
pub async fn purge(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;
    let group_header = compute_header(&entry);

    if group_header.is_dynamic {
        let group_name = attr_first(&entry, "name").unwrap_or_else(|| id.clone());
        let html = build_purge_modal(
            &id,
            &group_name,
            Some("Dynamic group membership is computed automatically and cannot be edited.".to_string()),
        );
        return Ok(Html(html).into_response());
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client.idm_group_purge_members(&id).await {
        Ok(()) => {
            // Re-fetch and return updated members list
            let entry = fetch_group(&state, &user, &id).await?;
            let group = compute_header(&entry);
            let data = build_members_data(&state, &user, &id, &entry).await;
            let html = askama::Template::render(&MembersListFragment {
                data: &data,
                group: &group,
            })
            .map_err(AppError::Template)?;
            // Also close the modal overlay
            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Trigger",
                "closeModal".parse().expect("static header"),
            );
            Ok((headers, Html(html)).into_response())
        }
        Err(e) => {
            tracing::warn!(group = %id, error = ?e, "purge members failed");
            let msg = friendly_error("purge members", &e);
            // Return error modal
            let entry = fetch_group(&state, &user, &id).await?;
            let group_name = attr_first(&entry, "name").unwrap_or_else(|| id.clone());
            let html = build_purge_modal(&id, &group_name, Some(msg));
            Ok(Html(html).into_response())
        }
    }
}

fn build_purge_modal(id: &str, group_name: &str, error: Option<String>) -> String {
    let input_id = format!("purge-{id}");
    let initials: String = group_name.chars().take(2).collect::<String>().to_uppercase();

    let target_html = IdentityRow {
        initials,
        displayname: group_name.to_string(),
        spn: group_name.to_string(),
    }
    .render()
    .unwrap_or_default();

    let confirm_token_js =
        serde_json::to_string(group_name).unwrap_or_else(|_| format!("{:?}", group_name));

    let body_html = DestructiveConfirm {
        lead_text: "This will remove ALL members from:".to_string(),
        target_html,
        consequences: vec![
            "All static members will be removed immediately.".to_string(),
            "This cannot be undone — you'll need to re-add members manually.".to_string(),
        ],
        confirm_token: group_name.to_string(),
        confirm_token_js,
        confirm_label: "Type the group name to confirm:".to_string(),
        input_id: input_id.clone(),
        error,
    }
    .render()
    .unwrap_or_default();

    let footer_html = DeleteFooter {
        action_url: format!("/groups/{id}/members/purge"),
        confirm_label: "Purge all members".to_string(),
        input_id,
        hx_vals_json: None,
    }
    .render()
    .unwrap_or_default();

    Modal {
        title: "Purge all members".to_string(),
        icon_name: Some("shield-alert"),
        icon_color_class: "text-danger",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .unwrap_or_default()
}

/// Returns the purge confirmation modal HTML (for triggering via HTMX get).
pub async fn purge_modal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;
    let group_name = attr_first(&entry, "name").unwrap_or_else(|| id.clone());
    let html = build_purge_modal(&id, &group_name, None);
    Ok(Html(html).into_response())
}

#[cfg(test)]
mod tests {
    use super::encode_member_id;

    #[test]
    fn at_sign_encodes_to_percent40() {
        assert_eq!(encode_member_id("@"), "%40");
    }

    #[test]
    fn slash_encodes_to_percent2f() {
        assert_eq!(encode_member_id("/"), "%2F");
    }

    #[test]
    fn alphanumeric_and_unreserved_passthrough() {
        assert_eq!(encode_member_id("abc-XYZ_1.2~"), "abc-XYZ_1.2~");
    }

    #[test]
    fn multibyte_utf8_encodes_correctly() {
        assert_eq!(encode_member_id("é"), "%C3%A9");
    }

    #[test]
    fn empty_string_stays_empty() {
        assert_eq!(encode_member_id(""), "");
    }

    #[test]
    fn spn_encodes_at_sign() {
        assert_eq!(encode_member_id("alice@example.com"), "alice%40example.com");
    }
}

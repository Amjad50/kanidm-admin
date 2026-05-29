use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::friendly_client_error;
use crate::kanidm::entry::attr_all;
use crate::views::dropdown::{DropdownItem, render_actions_cell};

use super::detail::{TabContent, compute_header, fetch_person, render_detail};

// ── Data model ────────────────────────────────────────────────────────────────

pub struct GroupRow {
    pub name: String,
    pub spn_or_id: String,
    pub initials: String,
    pub actions_html: String,
}

pub struct GroupsTabData {
    pub person_id: String,
    pub direct: Vec<GroupRow>,
    pub indirect_count: usize,
    pub error: Option<String>,
    /// SPNs of all groups, for the "add to group" datalist typeahead.
    pub all_group_spns: Vec<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_groups(entry: &kanidm_proto::v1::Entry) -> (Vec<String>, usize) {
    let direct: Vec<String> = attr_all(entry, "directmemberof");
    let transitive: Vec<String> = attr_all(entry, "memberof");
    let indirect = transitive.iter().filter(|g| !direct.contains(g)).count();
    (direct, indirect)
}

fn group_initials(name: &str) -> String {
    // Group names rarely have spaces; take the first two characters.
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn build_group_row(spn: &str, person_id: &str) -> GroupRow {
    // SPN form is "name@domain". The bare name comes before the '@'.
    let name = spn.split('@').next().unwrap_or(spn).to_string();
    let initials = group_initials(&name);

    // Person id may contain @, percent-encode it for the URL slot.
    let encoded_person = encode_path_segment(person_id);

    let actions_html = render_actions_cell(
        vec![
            DropdownItem::htmx_post(
                "Remove from group",
                format!("/admin/groups/{spn}/members/{encoded_person}/remove"),
            )
            .with_icon("x")
            .with_target("#person-groups-tab")
            .with_swap("outerHTML")
            .with_confirm(format!("Remove this person from \"{name}\"?"))
            .danger(),
        ],
        format!("Remove from {name}"),
    );

    GroupRow {
        name,
        spn_or_id: spn.to_string(),
        initials,
        actions_html,
    }
}

fn encode_path_segment(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => vec![c],
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

async fn build_tab_data(
    state: &AppState,
    user: &AdminUser,
    entry: &kanidm_proto::v1::Entry,
    person_id: &str,
    error: Option<String>,
) -> GroupsTabData {
    let (direct, indirect_count) = parse_groups(entry);
    let direct_set: std::collections::HashSet<&String> = direct.iter().collect();
    let rows = direct
        .iter()
        .map(|spn| build_group_row(spn, person_id))
        .collect();

    // Fetch all group SPNs for the "add to group" datalist. Exclude groups
    // the person is already in. Best-effort: empty list on failure.
    let all_group_spns: Vec<String> = match state.kanidm.for_token(&user.token).await {
        Ok(client) => match client.idm_group_list().await {
            Ok(entries) => entries
                .iter()
                .filter_map(|e| crate::kanidm::entry::attr_first(e, "spn"))
                .filter(|spn| !direct_set.contains(spn))
                .collect(),
            Err(e) => {
                tracing::warn!(error = ?e, "failed to list groups for typeahead");
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    };

    GroupsTabData {
        person_id: person_id.to_string(),
        direct: rows,
        indirect_count,
        error,
        all_group_spns,
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let data = build_tab_data(&state, &user, &entry, &id, None).await;
    render_detail(is_htmx, user, person, "groups", TabContent::Groups(data))
}

// ── Add to group ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AddGroupForm {
    pub group: String,
}

/// POST /people/{id}/groups/add
///
/// Add this person to the group given in the form. Returns the rebuilt
/// Person Groups tab fragment (outerHTML swap of #person-groups-tab).
pub async fn add(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    axum::extract::Form(form): axum::extract::Form<AddGroupForm>,
) -> AppResult<Response> {
    let group_id = form.group.trim().to_string();

    let mut error: Option<String> = None;

    if group_id.is_empty() {
        error = Some("Pick a group.".to_string());
    } else {
        let client = state
            .kanidm
            .for_token(&user.token)
            .await
            .map_err(|e| AppError::Kanidm(e.to_string()))?;
        if let Err(e) = client
            .idm_group_add_members(&group_id, &[id.as_str()])
            .await
        {
            tracing::warn!(group = %group_id, person = %id, error = ?e, "add to group failed");
            error = Some(friendly_client_error("add to group", &e));
        }
    }

    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let data = build_tab_data(&state, &user, &entry, &id, error).await;

    use super::detail::TabContentFragment;
    let html = TabContentFragment {
        tab_content: &TabContent::Groups(data),
        person: &person,
    }
    .render()
    .map_err(AppError::Template)?;
    Ok(Html(html).into_response())
}

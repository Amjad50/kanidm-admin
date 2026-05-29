use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;
use kanidm_proto::v1::{UatPurposeStatus, UatStatus, UatStatusState};
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

/// `?show_inactive=1` query opt-in to include revoked + past-expiry sessions
/// in the list. Threaded through tab GET + bulk-action POSTs so HTMX swaps
/// preserve the toggle state.
#[derive(Debug, Deserialize, Default)]
pub struct ShowInactiveQuery {
    #[serde(default)]
    pub show_inactive: Option<String>,
}

impl ShowInactiveQuery {
    pub fn enabled(&self) -> bool {
        matches!(self.show_inactive.as_deref(), Some("1" | "true" | "yes"))
    }
}

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::views::sessions_card::SessionsCard;
use crate::views::{format_absolute, format_relative_future, format_relative_past};
use crate::AppState;

use super::common::friendly_client_error;
use super::detail::{compute_header, fetch_person, render_detail, TabContent};

// ── View model ────────────────────────────────────────────────────────────────

pub enum ExpiryDisplay {
    Never,
    Revoked,
    ExpiresAt { relative: String, absolute: String },
    Expired { absolute: String },
}

pub enum PurposeDisplay {
    ReadOnly,
    ReadWrite,
    Privileged,
}

pub struct SessionRow {
    pub session_id_short: String,
    pub session_id_full: String,
    pub issued_at_relative: String,
    pub issued_at_absolute: String,
    pub expiry_state: ExpiryDisplay,
    pub purpose: PurposeDisplay,
    /// Pre-formatted per-row revoke endpoint, e.g.
    /// `/admin/people/alice@example.com/sessions/{uuid}/destroy` or
    /// `/me/sessions/{uuid}/destroy`. Templates emit this directly.
    pub revoke_url: String,
}

/// Tab-content variant for the admin Sessions tab. The shared partial is
/// pre-rendered to HTML by the handler; the `_tab_sessions.html` template
/// just emits it via `|safe`.
pub struct SessionsData {
    pub card_html: String,
}

// ── Builders ──────────────────────────────────────────────────────────────────

/// Convert a `UatStatus` from kanidm into the shared `SessionRow` view model.
///
/// `revoke_url_prefix` is the URL stem to which `"/{session_id}/destroy"` is
/// appended — e.g. `"/admin/people/alice@example.com/sessions"` (admin) or
/// `"/me/sessions"` (self). `revoke_url_suffix` is appended after `/destroy`
/// — typically empty, or `"?show_inactive=1"` when the toggle is on so the
/// post-revoke re-render preserves the current view. Pre-computing the
/// per-row URL avoids in-template `format!` and keeps the partial agnostic
/// of the calling context.
pub fn build_session_row(
    uat: UatStatus,
    revoke_url_prefix: &str,
    revoke_url_suffix: &str,
) -> SessionRow {
    let id_str = uat.session_id.to_string();
    let session_id_short = id_str.chars().take(8).collect();

    let issued_at_relative = format_relative_past(uat.issued_at);
    let issued_at_absolute = format_absolute(uat.issued_at);

    let expiry_state = match uat.state {
        UatStatusState::NeverExpires => ExpiryDisplay::Never,
        UatStatusState::Revoked => ExpiryDisplay::Revoked,
        UatStatusState::ExpiresAt(exp) => {
            let now = OffsetDateTime::now_utc();
            if exp <= now {
                ExpiryDisplay::Expired {
                    absolute: format_absolute(exp),
                }
            } else {
                ExpiryDisplay::ExpiresAt {
                    relative: format_relative_future(exp),
                    absolute: format_absolute(exp),
                }
            }
        }
    };

    let purpose = match uat.purpose {
        UatPurposeStatus::ReadOnly => PurposeDisplay::ReadOnly,
        UatPurposeStatus::ReadWrite => PurposeDisplay::ReadWrite,
        UatPurposeStatus::PrivilegeCapable => PurposeDisplay::Privileged,
    };

    let revoke_url = format!(
        "{}/{}/destroy{}",
        revoke_url_prefix, id_str, revoke_url_suffix,
    );

    SessionRow {
        session_id_short,
        session_id_full: id_str,
        issued_at_relative,
        issued_at_absolute,
        expiry_state,
        purpose,
        revoke_url,
    }
}

/// True iff a `UatStatusState` represents a session that's already inactive
/// (revoked or past-expiry). Used by both contexts' "Clear expired/revoked"
/// handlers to filter the list before iterating destroy calls.
pub fn is_dead_state(state: &UatStatusState, now: OffsetDateTime) -> bool {
    match state {
        UatStatusState::Revoked => true,
        UatStatusState::ExpiresAt(exp) => *exp <= now,
        UatStatusState::NeverExpires => false,
    }
}

async fn fetch_sessions(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    revoke_url_prefix: &str,
    show_inactive: bool,
) -> (Vec<SessionRow>, Option<String>) {
    let client = match state.kanidm.for_token(&user.token).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "failed to build kanidm client for sessions");
            return (vec![], Some(format!("Could not connect to kanidm: {e:?}")));
        }
    };

    match client.idm_account_list_user_auth_token(id).await {
        Ok(mut list) => {
            // Filter dead rows (revoked + past-expiry) unless show_inactive
            // is on. Kanidm's DELETE on a UAT just flips state to Revoked —
            // the entry stays listed forever, so showing them by default is
            // pure noise (nothing actionable left).
            if !show_inactive {
                let now = OffsetDateTime::now_utc();
                list.retain(|uat| !is_dead_state(&uat.state, now));
            }
            // Newest first.
            list.sort_by_key(|uat| std::cmp::Reverse(uat.issued_at));
            let suffix = if show_inactive { "?show_inactive=1" } else { "" };
            let rows = list
                .into_iter()
                .map(|uat| build_session_row(uat, revoke_url_prefix, suffix))
                .collect();
            (rows, None)
        }
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "failed to list sessions");
            let msg = friendly_client_error("list sessions", &e);
            (vec![], Some(msg))
        }
    }
}

/// Build the admin-context `SessionsCard` for a given person id.
///
/// `show_inactive` should match the toggle currently driving the row list —
/// it propagates into the bulk-revoke URL so HTMX swaps keep the same view.
fn build_admin_card(
    rows: Vec<SessionRow>,
    error: Option<String>,
    person_id: &str,
    show_inactive: bool,
) -> SessionsCard {
    let suffix = if show_inactive { "?show_inactive=1" } else { "" };
    SessionsCard {
        rows,
        error,
        hx_target_id: "tab-content".to_string(),
        bulk_revoke_url: format!(
            "/admin/people/{}/sessions/destroy_all{}",
            person_id, suffix,
        ),
        bulk_revoke_label: "Destroy all".to_string(),
        bulk_revoke_confirm:
            "Destroy all sessions for this person? They will be signed out everywhere.".to_string(),
        revoke_row_confirm:
            "Destroy this session? The person will be signed out on that device.".to_string(),
        empty_subtitle: "Where this person is currently signed in.".to_string(),
        current_session_id: None,
        show_inactive,
        show_inactive_url: if show_inactive {
            format!("/admin/people/{}/sessions", person_id)
        } else {
            format!("/admin/people/{}/sessions?show_inactive=1", person_id)
        },
    }
}

fn render_admin_card(card: &SessionsCard) -> AppResult<String> {
    askama::Template::render(card).map_err(AppError::Template)
}

fn render_sessions_fragment(
    person: super::detail::PersonHeader,
    tab_content: TabContent,
) -> AppResult<Response> {
    use super::detail::TabContentFragment;

    let html = askama::Template::render(&TabContentFragment {
        tab_content: &tab_content,
        person: &person,
    })
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

// ── GET /people/{id}/sessions ─────────────────────────────────────────────────

pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    Query(q): Query<ShowInactiveQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let show_inactive = q.enabled();
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);

    let prefix = format!("/admin/people/{}/sessions", id);
    let (sessions, error) = fetch_sessions(&state, &user, &id, &prefix, show_inactive).await;

    let card = build_admin_card(sessions, error, &id, show_inactive);
    let card_html = render_admin_card(&card)?;

    let tab_content = TabContent::Sessions(SessionsData { card_html });

    render_detail(is_htmx, user, person, "sessions", tab_content)
}

// ── POST /people/{id}/sessions/{session_id}/destroy ───────────────────────────

pub async fn destroy_one(
    State(state): State<AppState>,
    Path((id, session_uuid)): Path<(String, Uuid)>,
    Query(q): Query<ShowInactiveQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let show_inactive = q.enabled();
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let destroy_error = match client
        .idm_account_destroy_user_auth_token(&id, session_uuid)
        .await
    {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(person = %id, session = %session_uuid, error = ?e, "session destroy failed");
            Some(friendly_client_error("destroy session", &e))
        }
    };

    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);

    let prefix = format!("/admin/people/{}/sessions", id);
    let (sessions, fetch_error) = fetch_sessions(&state, &user, &id, &prefix, show_inactive).await;
    let error = destroy_error.or(fetch_error);

    let card = build_admin_card(sessions, error, &id, show_inactive);
    let card_html = render_admin_card(&card)?;

    render_sessions_fragment(person, TabContent::Sessions(SessionsData { card_html }))
}

// ── POST /people/{id}/sessions/destroy_all ────────────────────────────────────

pub async fn destroy_all(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<ShowInactiveQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let show_inactive = q.enabled();
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let list = match client.idm_account_list_user_auth_token(&id).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "failed to list sessions for destroy-all");
            let msg = friendly_client_error("list sessions", &e);
            let entry = fetch_person(&state, &user, &id).await?;
            let person = compute_header(&entry);
            let card = build_admin_card(vec![], Some(msg), &id, show_inactive);
            let card_html = render_admin_card(&card)?;
            return render_sessions_fragment(
                person,
                TabContent::Sessions(SessionsData { card_html }),
            );
        }
    };

    let mut errors: Vec<String> = vec![];
    for uat in list {
        if let Err(e) = client
            .idm_account_destroy_user_auth_token(&id, uat.session_id)
            .await
        {
            tracing::warn!(person = %id, session = %uat.session_id, error = ?e, "session destroy failed during destroy-all");
            errors.push(friendly_client_error("destroy session", &e));
        }
    }

    let combined_error = if errors.is_empty() {
        None
    } else {
        Some(format!("Some sessions could not be destroyed: {}", errors.join("; ")))
    };

    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let prefix = format!("/admin/people/{}/sessions", id);
    let (sessions, fetch_error) = fetch_sessions(&state, &user, &id, &prefix, show_inactive).await;
    let error = combined_error.or(fetch_error);

    let card = build_admin_card(sessions, error, &id, show_inactive);
    let card_html = render_admin_card(&card)?;

    render_sessions_fragment(person, TabContent::Sessions(SessionsData { card_html }))
}


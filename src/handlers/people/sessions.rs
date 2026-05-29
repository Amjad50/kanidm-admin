use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;
use kanidm_proto::v1::{UatPurposeStatus, UatStatusState};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
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
}

pub struct SessionsData {
    pub person_id: String,
    pub sessions: Vec<SessionRow>,
    pub error: Option<String>,
}

// ── Builders ──────────────────────────────────────────────────────────────────

fn build_session_row(uat: kanidm_proto::v1::UatStatus) -> SessionRow {
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

    SessionRow {
        session_id_short,
        session_id_full: id_str,
        issued_at_relative,
        issued_at_absolute,
        expiry_state,
        purpose,
    }
}

async fn fetch_sessions(
    state: &AppState,
    user: &AdminUser,
    id: &str,
) -> (Vec<SessionRow>, Option<String>) {
    let client = match state.kanidm.for_token(&user.token).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "failed to build kanidm client for sessions");
            return (vec![], Some(format!("Could not connect to kanidm: {e:?}")));
        }
    };

    match client.idm_account_list_user_auth_token(id).await {
        Ok(list) => {
            let rows = list.into_iter().map(build_session_row).collect();
            (rows, None)
        }
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "failed to list sessions");
            let msg = friendly_client_error("list sessions", &e);
            (vec![], Some(msg))
        }
    }
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
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);

    let (sessions, error) = fetch_sessions(&state, &user, &id).await;

    let tab_content = TabContent::Sessions(SessionsData {
        person_id: id,
        sessions,
        error,
    });

    render_detail(is_htmx, user, person, "sessions", tab_content)
}

// ── POST /people/{id}/sessions/{session_id}/destroy ───────────────────────────

pub async fn destroy_one(
    State(state): State<AppState>,
    Path((id, session_uuid)): Path<(String, Uuid)>,
    user: AdminUser,
) -> AppResult<Response> {
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

    let (sessions, fetch_error) = fetch_sessions(&state, &user, &id).await;
    let error = destroy_error.or(fetch_error);

    render_sessions_fragment(
        person,
        TabContent::Sessions(SessionsData {
            person_id: id,
            sessions,
            error,
        }),
    )
}

// ── POST /people/{id}/sessions/destroy_all ────────────────────────────────────

pub async fn destroy_all(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
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
            return render_sessions_fragment(
                person,
                TabContent::Sessions(SessionsData {
                    person_id: id,
                    sessions: vec![],
                    error: Some(msg),
                }),
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
    let (sessions, fetch_error) = fetch_sessions(&state, &user, &id).await;
    let error = combined_error.or(fetch_error);

    render_sessions_fragment(
        person,
        TabContent::Sessions(SessionsData {
            person_id: id,
            sessions,
            error,
        }),
    )
}

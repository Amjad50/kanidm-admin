use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_htmx::HxRequest;
use kanidm_proto::v1::{UatPurposeStatus, UatStatusState};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::people::sessions::{ExpiryDisplay, PurposeDisplay, SessionRow};
use crate::kanidm::entry::{attr_all, attr_first, attr_present};
use crate::views::{format_absolute, format_relative_future, format_relative_past, initials, BaseFields};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(profile))
        .route("/me/sessions", get(sessions_tab))
        .route(
            "/me/sessions/{session_id}/destroy",
            axum::routing::post(destroy_session),
        )
}

// ── Profile view ─────────────────────────────────────────────────────────────

pub struct GroupChip {
    pub name: String,
    pub spn_or_id: String,
}

pub struct CurrentSession {
    pub signed_in_relative: Option<String>,
    pub expires_relative: Option<String>,
    pub privileged: bool,
    pub privileged_remaining: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "self_user/profile.html")]
pub struct ProfileView {
    pub base: BaseFields,
    pub avatar_initials: String,
    pub displayname: String,
    pub spn: String,
    pub uuid: String,
    pub name: String,
    pub legalname: Option<String>,
    pub primary_mail: Option<String>,
    pub mails: Vec<String>,
    pub groups: Vec<GroupChip>,
    pub direct_group_count: usize,
    pub current_session: CurrentSession,
}

pub async fn profile(State(state): State<AppState>, user: AdminUser) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let entry = client
        .whoami()
        .await
        .map_err(|e| AppError::Kanidm(format!("whoami failed: {e:?}")))?
        .ok_or_else(|| AppError::Kanidm("whoami returned no entry".to_string()))?;

    let displayname = attr_first(&entry, "displayname")
        .or_else(|| attr_first(&entry, "name"))
        .unwrap_or_default();
    let spn = attr_first(&entry, "spn").unwrap_or_default();
    let uuid_str = attr_first(&entry, "uuid").unwrap_or_default();
    let name = attr_first(&entry, "name").unwrap_or_default();
    let legalname = attr_first(&entry, "legalname");
    let mails = attr_all(&entry, "mail");
    let primary_mail = mails.first().cloned();

    let group_spns = if attr_present(&entry, "directmemberof") {
        attr_all(&entry, "directmemberof")
    } else {
        attr_all(&entry, "memberof")
    };
    let direct_group_count = group_spns.len();
    let groups: Vec<GroupChip> = group_spns
        .into_iter()
        .take(8)
        .map(|spn| {
            let n = spn.split('@').next().unwrap_or(&spn).to_string();
            GroupChip { name: n, spn_or_id: spn }
        })
        .collect();

    let current_session = CurrentSession {
        signed_in_relative: user.signed_in_at.map(format_relative_past),
        expires_relative: user.session_expires_at.map(format_relative_future),
        privileged: user.privileged,
        privileged_remaining: user.privileged_until.map(format_relative_future),
    };

    let avatar_initials = initials(&displayname);

    let view = ProfileView {
        base: BaseFields::new(&user, "me"),
        avatar_initials,
        displayname,
        spn,
        uuid: uuid_str,
        name,
        legalname,
        primary_mail,
        mails,
        groups,
        direct_group_count,
        current_session,
    };

    Ok(view.into_response())
}

// ── Sessions ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "self_user/sessions.html")]
pub struct SessionsView {
    pub base: BaseFields,
    pub displayname: String,
    pub spn: String,
    pub sessions: Vec<SessionRow>,
    pub current_session_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "self_user/_sessions_table.html")]
pub struct SessionsTableFragment {
    pub sessions: Vec<SessionRow>,
    pub current_session_id: Option<String>,
    pub error: Option<String>,
}

fn build_row(uat: kanidm_proto::v1::UatStatus) -> SessionRow {
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

async fn fetch_my_sessions(
    state: &AppState,
    user: &AdminUser,
) -> (Vec<SessionRow>, Option<String>) {
    let client = match state.kanidm.for_token(&user.token).await {
        Ok(c) => c,
        Err(e) => {
            return (vec![], Some(format!("Could not connect to kanidm: {e:?}")));
        }
    };

    match client.idm_account_list_user_auth_token(&user.spn).await {
        Ok(list) => (list.into_iter().map(build_row).collect(), None),
        Err(e) => {
            tracing::warn!(spn = %user.spn, error = ?e, "self session list failed");
            let msg = crate::handlers::common::friendly_client_error("list your sessions", &e);
            (vec![], Some(msg))
        }
    }
}

fn current_session_id_from_token(token: &str) -> Option<String> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    v.get("session_id").and_then(|s| s.as_str()).map(|s| s.to_string())
}

pub async fn sessions_tab(
    State(state): State<AppState>,
    user: AdminUser,
) -> AppResult<Response> {
    let (sessions, error) = fetch_my_sessions(&state, &user).await;
    let current_session_id = current_session_id_from_token(&user.token);

    Ok(SessionsView {
        base: BaseFields::new(&user, "me"),
        displayname: user.displayname.clone(),
        spn: user.spn.clone(),
        sessions,
        current_session_id,
        error,
    }
    .into_response())
}

pub async fn destroy_session(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    user: AdminUser,
) -> AppResult<Response> {
    let current_session_id = current_session_id_from_token(&user.token);

    // Don't let the user destroy their own active session — they'd be logged out
    // immediately. They can use Logout for that.
    if let Some(cur) = &current_session_id {
        if cur == &session_id.to_string() {
            let (sessions, _err) = fetch_my_sessions(&state, &user).await;
            let fragment = SessionsTableFragment {
                sessions,
                current_session_id,
                error: Some(
                    "Cannot destroy the session you are currently using — use Log out instead."
                        .to_string(),
                ),
            };
            return Ok(Html(askama::Template::render(&fragment).map_err(AppError::Template)?)
                .into_response());
        }
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let destroy_error = match client
        .idm_account_destroy_user_auth_token(&user.spn, session_id)
        .await
    {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(spn = %user.spn, session = %session_id, error = ?e, "self session destroy failed");
            Some(crate::handlers::common::friendly_client_error("destroy session", &e))
        }
    };

    let (sessions, fetch_error) = fetch_my_sessions(&state, &user).await;
    let error = destroy_error.or(fetch_error);

    if is_htmx {
        let fragment = SessionsTableFragment {
            sessions,
            current_session_id,
            error,
        };
        return Ok(Html(askama::Template::render(&fragment).map_err(AppError::Template)?)
            .into_response());
    }

    Ok(SessionsView {
        base: BaseFields::new(&user, "me"),
        displayname: user.displayname.clone(),
        spn: user.spn.clone(),
        sessions,
        current_session_id,
        error,
    }
    .into_response())
}

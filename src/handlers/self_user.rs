use askama::Template;
use askama_web::WebTemplate;
use axum::Router;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum_htmx::HxRequest;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::people::sessions::{
    SessionRow, ShowInactiveQuery, build_session_row, is_dead_state,
};
use crate::kanidm::entry::{attr_all, attr_first, attr_present};
use crate::views::sessions_card::SessionsCard;
use crate::views::{BaseFields, format_relative_future, format_relative_past, initials};

/// URL prefix passed to `build_session_row` so per-row revoke URLs come out
/// as `/admin/me/sessions/{uuid}/destroy`. Lifted to a constant so all the
/// call sites stay in sync.
const SELF_SESSIONS_PREFIX: &str = "/admin/me/sessions";

pub fn router() -> Router<AppState> {
    // Route literals are relative; the parent router nests this under /admin.
    Router::new()
        .route("/me", get(profile))
        .route("/me/sessions", get(sessions_tab))
        .route(
            "/me/sessions/{session_id}/destroy",
            axum::routing::post(destroy_session),
        )
        .route(
            "/me/sessions/destroy_others",
            axum::routing::post(destroy_others),
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
            GroupChip {
                name: n,
                spn_or_id: spn,
            }
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

/// Top-level page for `/me/sessions`. Wraps the shared sessions card in
/// page chrome (title, crumb). The card itself is pre-rendered in the
/// handler and embedded via `{{ card_html|safe }}`.
#[derive(Template, WebTemplate)]
#[template(path = "self_user/sessions.html")]
pub struct SessionsView {
    pub base: BaseFields,
    pub displayname: String,
    pub card_html: String,
}

/// Build the self-context `SessionsCard`. Buttons swap `#sessions-table`
/// (the wrapper div in `sessions.html`); the bulk-revoke action targets all
/// sessions other than the viewer's current one.
fn build_self_card(
    rows: Vec<SessionRow>,
    error: Option<String>,
    current_session_id: Option<String>,
    show_inactive: bool,
) -> SessionsCard {
    let suffix = if show_inactive {
        "?show_inactive=1"
    } else {
        ""
    };
    SessionsCard {
        rows,
        error,
        hx_target_id: "sessions-table".to_string(),
        bulk_revoke_url: format!("/admin/me/sessions/destroy_others{}", suffix),
        bulk_revoke_label: "Destroy other sessions".to_string(),
        bulk_revoke_confirm:
            "Destroy every session except the one you are currently using? Those other devices will be signed out immediately.".to_string(),
        revoke_row_confirm:
            "Destroy this session? The device using it will be signed out.".to_string(),
        empty_subtitle: "Sign out remotely by revoking a session.".to_string(),
        current_session_id,
        show_inactive,
        show_inactive_url: if show_inactive {
            "/admin/me/sessions".to_string()
        } else {
            "/admin/me/sessions?show_inactive=1".to_string()
        },
    }
}

fn render_self_card(card: &SessionsCard) -> AppResult<String> {
    askama::Template::render(card).map_err(AppError::Template)
}

async fn fetch_my_sessions(
    state: &AppState,
    user: &AdminUser,
    show_inactive: bool,
) -> (Vec<SessionRow>, Option<String>) {
    let client = match state.kanidm.for_token(&user.token).await {
        Ok(c) => c,
        Err(e) => {
            return (vec![], Some(format!("Could not connect to kanidm: {e:?}")));
        }
    };

    match client.idm_account_list_user_auth_token(&user.spn).await {
        Ok(mut list) => {
            // Filter dead rows + newest first. See the matching comment in
            // people/sessions.rs::fetch_sessions for the rationale.
            if !show_inactive {
                let now = OffsetDateTime::now_utc();
                list.retain(|uat| !is_dead_state(&uat.state, now));
            }
            list.sort_by_key(|uat| std::cmp::Reverse(uat.issued_at));
            let suffix = if show_inactive {
                "?show_inactive=1"
            } else {
                ""
            };
            (
                list.into_iter()
                    .map(|uat| build_session_row(uat, SELF_SESSIONS_PREFIX, suffix))
                    .collect(),
                None,
            )
        }
        Err(e) => {
            tracing::warn!(spn = %user.spn, error = ?e, "self session list failed");
            let msg = crate::handlers::common::friendly_client_error("list your sessions", &e);
            (vec![], Some(msg))
        }
    }
}

pub async fn sessions_tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Query(q): Query<ShowInactiveQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let show_inactive = q.enabled();
    let (sessions, error) = fetch_my_sessions(&state, &user, show_inactive).await;
    let current_session_id = user.session_id.clone();

    respond_self_sessions(
        is_htmx,
        &user,
        sessions,
        error,
        current_session_id,
        show_inactive,
    )
}

/// Render either the full page (non-HTMX) or just the inner card fragment
/// (HTMX swap targeting `#sessions-table`).
fn respond_self_sessions(
    is_htmx: bool,
    user: &AdminUser,
    sessions: Vec<SessionRow>,
    error: Option<String>,
    current_session_id: Option<String>,
    show_inactive: bool,
) -> AppResult<Response> {
    let card = build_self_card(sessions, error, current_session_id, show_inactive);
    let card_html = render_self_card(&card)?;

    if is_htmx {
        return Ok(Html(card_html).into_response());
    }

    Ok(SessionsView {
        base: BaseFields::new(user, "me"),
        displayname: user.displayname.clone(),
        card_html,
    }
    .into_response())
}

pub async fn destroy_session(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Query(q): Query<ShowInactiveQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let show_inactive = q.enabled();
    let current_session_id = user.session_id.clone();

    // Don't let the user destroy their own active session — they'd be logged
    // out immediately. They can use Logout for that. The shared partial also
    // disables this button on the current-session row, so this branch only
    // trips if a client hand-crafts the request.
    if let Some(cur) = &current_session_id
        && cur == &session_id.to_string()
    {
        let (sessions, _err) = fetch_my_sessions(&state, &user, show_inactive).await;
        return respond_self_sessions(
            is_htmx,
            &user,
            sessions,
            Some(
                "Cannot destroy the session you are currently using — use Log out instead."
                    .to_string(),
            ),
            current_session_id,
            show_inactive,
        );
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
            Some(crate::handlers::common::friendly_client_error(
                "destroy session",
                &e,
            ))
        }
    };

    let (sessions, fetch_error) = fetch_my_sessions(&state, &user, show_inactive).await;
    let error = destroy_error.or(fetch_error);

    respond_self_sessions(
        is_htmx,
        &user,
        sessions,
        error,
        current_session_id,
        show_inactive,
    )
}

// ── POST /me/sessions/destroy_others ─────────────────────────────────────────

/// Destroy every session for the current user *except* the one they are
/// currently using. Mirrors the admin context's "Destroy all", with the
/// current-session skip.
pub async fn destroy_others(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Query(q): Query<ShowInactiveQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let show_inactive = q.enabled();
    let current_session_id = user.session_id.clone();

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let list = match client.idm_account_list_user_auth_token(&user.spn).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(spn = %user.spn, error = ?e, "self session list failed (destroy_others)");
            let msg = crate::handlers::common::friendly_client_error("list your sessions", &e);
            return respond_self_sessions(
                is_htmx,
                &user,
                vec![],
                Some(msg),
                current_session_id,
                show_inactive,
            );
        }
    };

    let mut errors: Vec<String> = vec![];
    for uat in &list {
        let id_str = uat.session_id.to_string();
        if current_session_id.as_deref() == Some(id_str.as_str()) {
            continue;
        }
        if let Err(e) = client
            .idm_account_destroy_user_auth_token(&user.spn, uat.session_id)
            .await
        {
            tracing::warn!(spn = %user.spn, session = %uat.session_id, error = ?e, "destroy_others failed for session");
            errors.push(crate::handlers::common::friendly_client_error(
                "destroy session",
                &e,
            ));
        }
    }

    let combined_error = if errors.is_empty() {
        None
    } else {
        Some(format!(
            "Some sessions could not be destroyed: {}",
            errors.join("; ")
        ))
    };

    let (sessions, fetch_error) = fetch_my_sessions(&state, &user, show_inactive).await;
    let error = combined_error.or(fetch_error);

    respond_self_sessions(
        is_htmx,
        &user,
        sessions,
        error,
        current_session_id,
        show_inactive,
    )
}

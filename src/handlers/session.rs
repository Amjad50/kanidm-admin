use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::post;
use axum::Router;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use axum_htmx::HxRequest;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::AdminUser;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/logout", post(logout))
}

/// Destroy `session_id` on kanidm for `spn`. Logs and swallows errors —
/// callers (logout) want the cookie cleared even if the server call fails.
pub async fn destroy_session_best_effort(
    state: &AppState,
    token: &str,
    spn: &str,
    session_id: &str,
) {
    let Ok(parsed) = session_id.parse::<Uuid>() else {
        tracing::warn!(session_id, "skipping destroy: session_id not a valid UUID");
        return;
    };
    let client = match state.kanidm.for_token(token).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = ?e, "skipping destroy: could not build kanidm client");
            return;
        }
    };
    if let Err(e) = client.idm_account_destroy_user_auth_token(spn, parsed).await {
        tracing::warn!(spn, %session_id, error = ?e, "destroying current session on kanidm failed");
    }
}

pub async fn logout(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    jar: CookieJar,
    user: AdminUser,
) -> Response {
    if let Some(sid) = &user.session_id {
        destroy_session_best_effort(&state, &user.token, &user.spn, sid).await;
    }

    let cookie_name = state.config.kanidm_session_cookie.clone();
    let kanidm_url = state.config.kanidm_url.clone();

    let expired = Cookie::build((cookie_name, ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build();
    let jar = jar.add(expired);

    if is_htmx {
        let mut resp = StatusCode::OK.into_response();
        if let Ok(v) = HeaderValue::from_str(&kanidm_url) {
            resp.headers_mut().insert("HX-Redirect", v);
        }
        return (jar, resp).into_response();
    }
    (jar, Redirect::to(&kanidm_url)).into_response()
}

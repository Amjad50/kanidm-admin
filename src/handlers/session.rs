use axum::Router;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::post;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_htmx::HxRequest;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppState;
use crate::auth::AdminUser;

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
    if let Err(e) = client
        .idm_account_destroy_user_auth_token(spn, parsed)
        .await
    {
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

    let expired = Cookie::build((state.config.session_cookie_name.clone(), ""))
        .path("/")
        .http_only(true)
        .secure(!state.config.dev_insecure_cookies)
        .same_site(SameSite::Lax)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build();
    let jar = jar.add(expired);

    if is_htmx {
        let mut resp = StatusCode::OK.into_response();
        resp.headers_mut()
            .insert("HX-Redirect", HeaderValue::from_static("/login"));
        return (jar, resp).into_response();
    }
    (jar, Redirect::to("/login")).into_response()
}

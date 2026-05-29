use axum::extract::State;
use axum::response::Redirect;
use axum::routing::post;
use axum::Router;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::OffsetDateTime;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/logout", post(logout))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let cookie_name = state.config.kanidm_session_cookie.clone();
    let kanidm_url = state.config.kanidm_url.clone();

    // Build an expired cookie to clear the session token from the browser.
    let expired = Cookie::build((cookie_name, ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build();

    let jar = jar.add(expired);

    (jar, Redirect::to(&kanidm_url))
}

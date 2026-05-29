pub mod common;
mod dashboard;
mod empty;
mod groups;
mod health;
mod oauth2;
mod people;
mod reauth;
mod self_user;
mod session;

use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

use crate::views::NotFoundView;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/empty", get(empty::empty))
        .route("/reauth", get(reauth::reauth))
        .route("/", get(dashboard::dashboard))
        .merge(people::router())
        .merge(groups::router())
        .merge(oauth2::router())
        .merge(self_user::router())
        .merge(session::router())
}

pub async fn not_found() -> (StatusCode, NotFoundView) {
    (StatusCode::NOT_FOUND, NotFoundView {})
}

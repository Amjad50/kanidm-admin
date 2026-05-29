pub mod common;
mod dashboard;
mod empty;
mod groups;
mod health;
mod login;
mod oauth2;
pub(crate) mod people;
mod self_user;
mod session;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

use crate::AppState;
use crate::views::NotFoundView;

pub fn router() -> Router<AppState> {
    Router::new()
        // Root: utility + auth + user-facing
        .route("/healthz", get(health::healthz))
        .route("/empty", get(empty::empty))
        .merge(login::router()) // /login, /login/*
        .merge(session::router()) // /logout
        .merge(self_user::router()) // /me, /me/sessions
        // /admin/*: operator pages
        .nest(
            "/admin",
            Router::new()
                .route("/", get(dashboard::dashboard))
                .merge(people::router())
                .merge(groups::router())
                .merge(oauth2::router()),
        )
}

pub async fn not_found() -> (StatusCode, NotFoundView) {
    (StatusCode::NOT_FOUND, NotFoundView {})
}

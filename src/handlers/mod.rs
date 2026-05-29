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
use axum::response::Redirect;
use axum::routing::get;

use crate::AppState;
use crate::views::NotFoundView;

pub fn router() -> Router<AppState> {
    // The whole app lives under /admin/*. The reverse proxy can either send
    // a dedicated host (e.g. admin.idm.example.com → all paths) or just the
    // /admin/* paths on a shared host (e.g. idm.example.com/admin/*). The
    // root-level redirect is there so visiting the dedicated-host root
    // lands on the dashboard.
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/admin") }))
        .nest(
            "/admin",
            Router::new()
                .route("/", get(dashboard::dashboard))
                .route("/healthz", get(health::healthz))
                .route("/empty", get(empty::empty))
                .merge(login::router()) // /admin/login, /admin/login/*
                .merge(session::router()) // /admin/logout
                .merge(self_user::router()) // /admin/me, /admin/me/sessions
                .merge(people::router()) // /admin/people/*
                .merge(groups::router()) // /admin/groups/*
                .merge(oauth2::router()), // /admin/oauth2/*
        )
}

pub async fn not_found() -> (StatusCode, NotFoundView) {
    (StatusCode::NOT_FOUND, NotFoundView {})
}

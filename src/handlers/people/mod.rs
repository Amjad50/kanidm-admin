mod common;
pub mod create;
pub mod delete;
pub mod detail;
pub mod edit;
pub mod list;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/people", get(list::list).post(create::submit))
        .route("/people/new", get(create::create_form))
        // Detail routes — order matters: /new must be registered before /{id}
        .route("/people/{id}", get(detail::redirect_to_overview).post(edit::submit))
        .route("/people/{id}/overview", get(detail::overview))
        .route("/people/{id}/credentials", get(detail::credentials_tab))
        .route("/people/{id}/ssh", get(detail::ssh_tab))
        .route("/people/{id}/radius", get(detail::radius_tab))
        .route("/people/{id}/sessions", get(detail::sessions_tab))
        .route("/people/{id}/validity", get(detail::validity_tab))
        .route("/people/{id}/edit", get(edit::edit_form))
        .route("/people/{id}/delete", get(delete::show).post(delete::submit))
}

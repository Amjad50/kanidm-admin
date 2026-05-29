mod common;
pub mod create;
pub mod credentials;
pub mod delete;
pub mod detail;
pub mod edit;
pub mod list;
pub mod radius;
pub mod ssh;

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
        .route("/people/{id}/credentials", get(credentials::tab))
        .route("/people/{id}/credentials/reset",
            get(credentials::reset_modal).post(credentials::submit_reset))
        .route("/people/{id}/ssh", get(ssh::tab).post(ssh::add))
        .route("/people/{id}/ssh/{tag}/delete", axum::routing::post(ssh::delete))
        .route("/people/{id}/radius", get(radius::tab))
        .route("/people/{id}/radius/regenerate", axum::routing::post(radius::regenerate))
        .route("/people/{id}/radius/delete", axum::routing::post(radius::delete_secret))
        .route("/people/{id}/sessions", get(detail::sessions_tab))
        .route("/people/{id}/validity", get(detail::validity_tab))
        .route("/people/{id}/edit", get(edit::edit_form))
        .route("/people/{id}/delete", get(delete::show).post(delete::submit))
}

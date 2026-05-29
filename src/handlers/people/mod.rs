pub(crate) mod common;
pub mod create;
pub mod credentials;
pub mod delete;
pub mod detail;
pub mod edit;
pub mod list;
pub mod radius;
pub mod sessions;
pub mod ssh;
pub mod validity;

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
        .route("/people/{id}/sessions", get(sessions::tab))
        // destroy_all is a static segment so axum prioritizes it over the dynamic {session_id}/destroy.
        .route("/people/{id}/sessions/{session_id}/destroy", axum::routing::post(sessions::destroy_one))
        .route("/people/{id}/sessions/destroy_all", axum::routing::post(sessions::destroy_all))
        .route("/people/{id}/validity", get(validity::tab))
        .route("/people/{id}/validity/valid_from", axum::routing::post(validity::set_valid_from))
        .route("/people/{id}/validity/expire", axum::routing::post(validity::set_expire))
        .route("/people/{id}/edit", get(edit::edit_form))
        .route("/people/{id}/delete", get(delete::show).post(delete::submit))
}

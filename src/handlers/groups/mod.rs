mod common;
pub mod create;
pub mod delete;
pub mod detail;
pub mod edit;
pub mod list;
pub mod members;
pub mod policy;

use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/groups", get(list::list).post(create::submit))
        .route("/groups/new", get(create::create_form))
        // Detail — /new must come before /{id}
        .route(
            "/groups/{id}",
            get(detail::redirect_to_overview).post(edit::submit),
        )
        .route("/groups/{id}/overview", get(detail::overview))
        .route("/groups/{id}/members", get(members::tab))
        .route("/groups/{id}/members/add", post(members::add))
        .route(
            "/groups/{id}/members/purge-modal",
            get(members::purge_modal),
        )
        .route("/groups/{id}/members/{mid}/remove", post(members::remove))
        .route("/groups/{id}/members/purge", post(members::purge))
        .route("/groups/{id}/policy", get(policy::tab))
        .route("/groups/{id}/policy/{field}", post(policy::set_field))
        .route(
            "/groups/{id}/policy/{field}/reset",
            post(policy::reset_field),
        )
        .route("/groups/{id}/edit", get(edit::edit_form))
        .route(
            "/groups/{id}/delete",
            get(delete::show).post(delete::submit),
        )
}

pub(crate) mod common;
pub(super) mod create;
pub mod detail;
pub mod general;
pub mod list;
pub mod secret;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2", get(list::list).post(create::submit))
        // /new must come before /{id}
        .route("/oauth2/new", get(create::pick_type))
        .route("/oauth2/new/details", get(create::details_form))
        // Detail shell: /{id} → 308 to /{id}/general
        .route("/oauth2/{id}", get(detail::redirect_to_general))
        // General tab — GET + POST (toggle/field updates)
        .route("/oauth2/{id}/general", get(general::tab).post(general::update))
        // Redirect URL management
        .route("/oauth2/{id}/redirect/add", post(general::add_redirect))
        .route("/oauth2/{id}/redirect/{idx}/remove", post(general::remove_redirect))
        // Secret subscreen (not a tab)
        .route("/oauth2/{id}/secret",       get(secret::tab))
        .route("/oauth2/{id}/secret/reset", post(secret::reset))
        // Placeholder tabs — 4D–4I will replace each one
        .route("/oauth2/{id}/endpoints",  get(detail::endpoints_placeholder))
        .route("/oauth2/{id}/scope-maps", get(detail::scope_maps_placeholder))
        .route("/oauth2/{id}/claim-maps", get(detail::claim_maps_placeholder))
        .route("/oauth2/{id}/crypto",     get(detail::crypto_placeholder))
        .route("/oauth2/{id}/image",      get(detail::image_placeholder))
        .route("/oauth2/{id}/advanced",   get(detail::advanced_placeholder))
}

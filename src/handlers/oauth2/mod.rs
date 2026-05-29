pub mod advanced;
pub mod claim_maps;
pub(crate) mod common;
pub(super) mod create;
pub mod crypto;
pub mod delete;
pub mod detail;
pub mod general;
pub mod image;
pub mod list;
pub mod overview;
pub mod scope_maps;
pub mod secret;

use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2", get(list::list).post(create::submit))
        // /new must come before /{id}
        .route("/oauth2/new", get(create::pick_type))
        .route("/oauth2/new/details", get(create::details_form))
        // Detail shell: /{id} → 308 to /{id}/overview
        .route("/oauth2/{id}", get(detail::redirect_to_overview))
        // Overview tab — 4D
        .route("/oauth2/{id}/overview", get(overview::tab))
        // General tab — GET + POST (toggle/field updates)
        .route(
            "/oauth2/{id}/general",
            get(general::tab).post(general::update),
        )
        // Redirect URL management
        .route("/oauth2/{id}/redirect/add", post(general::add_redirect))
        .route(
            "/oauth2/{id}/redirect/{idx}/remove",
            post(general::remove_redirect),
        )
        // Secret subscreen (not a tab)
        .route("/oauth2/{id}/secret", get(secret::tab))
        .route("/oauth2/{id}/secret/reset", post(secret::reset))
        // Scope maps tab — 4E
        .route("/oauth2/{id}/scope-maps", get(scope_maps::tab))
        // Add modal — must come before /{group}/edit to avoid routing ambiguity
        .route(
            "/oauth2/{id}/scope-map/standard/new",
            get(scope_maps::standard_new_modal),
        )
        .route(
            "/oauth2/{id}/scope-map/supplementary/new",
            get(scope_maps::supplementary_new_modal),
        )
        .route(
            "/oauth2/{id}/scope-map/standard",
            post(scope_maps::add_standard),
        )
        .route(
            "/oauth2/{id}/scope-map/standard/{group}/edit",
            get(scope_maps::standard_edit_modal),
        )
        .route(
            "/oauth2/{id}/scope-map/standard/{group}/delete",
            post(scope_maps::delete_standard),
        )
        .route(
            "/oauth2/{id}/scope-map/supplementary",
            post(scope_maps::add_supplementary),
        )
        .route(
            "/oauth2/{id}/scope-map/supplementary/{group}/edit",
            get(scope_maps::supplementary_edit_modal),
        )
        .route(
            "/oauth2/{id}/scope-map/supplementary/{group}/delete",
            post(scope_maps::delete_supplementary),
        )
        // Claim maps tab — 4F
        .route("/oauth2/{id}/claim-maps", get(claim_maps::tab))
        .route("/oauth2/{id}/claim-map", post(claim_maps::add))
        .route(
            "/oauth2/{id}/claim-map/new",
            get(claim_maps::new_claim_modal),
        )
        .route(
            "/oauth2/{id}/claim-map/{claim}/{group}/edit",
            get(claim_maps::edit_row_modal),
        )
        .route(
            "/oauth2/{id}/claim-map/{claim}/{group}/delete",
            post(claim_maps::delete),
        )
        .route(
            "/oauth2/{id}/claim-map/{claim}/add-group",
            get(claim_maps::add_group_modal),
        )
        .route(
            "/oauth2/{id}/claim-map/{claim}/join-modal",
            get(claim_maps::join_strategy_modal),
        )
        .route(
            "/oauth2/{id}/claim-map/{claim}/join",
            post(claim_maps::set_join),
        )
        .route(
            "/oauth2/{id}/claim-map/{claim}/delete-all",
            post(claim_maps::delete_all_for_claim),
        )
        // Crypto tab — 4G
        .route("/oauth2/{id}/crypto", get(crypto::tab))
        .route("/oauth2/{id}/crypto/rotate", post(crypto::rotate))
        .route("/oauth2/{id}/crypto/revoke", post(crypto::revoke))
        .route(
            "/oauth2/{id}/crypto/revoke-modal",
            get(crypto::revoke_modal),
        )
        // Image tab — 4H
        .route("/oauth2/{id}/image", get(image::tab).post(image::upload))
        .route("/oauth2/{id}/image/from-url", post(image::upload_from_url))
        .route("/oauth2/{id}/image/delete", post(image::delete))
        .route("/oauth2/{id}/image-proxy", get(image::proxy))
        // Advanced tab — 4I
        .route(
            "/oauth2/{id}/advanced",
            get(advanced::tab).post(advanced::update),
        )
        // Delete modal — 4J
        .route(
            "/oauth2/{id}/delete",
            get(delete::show).post(delete::submit),
        )
}

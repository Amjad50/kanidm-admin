pub mod list;

use axum::extract::State;
use axum::routing::get;
use axum::Router;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::views::{BaseFields, PlaceholderView};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2", get(list::list))
        .route("/oauth2/new", get(create_placeholder))
        // /new must come before /{id}
        .route("/oauth2/{id}", get(detail_placeholder))
}

pub async fn create_placeholder(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "oauth2"),
        section_label: "New OAuth2 App",
        message: "Register a new OAuth2 resource server in your Kanidm instance.",
        phase_label: "Phase 4",
    })
}

pub async fn detail_placeholder(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "oauth2"),
        section_label: "OAuth2 App Detail",
        message: "View and edit an OAuth2 resource server in your Kanidm instance.",
        phase_label: "Phase 4",
    })
}

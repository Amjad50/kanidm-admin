use axum::extract::State;
use axum::routing::get;
use axum::Router;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::views::{BaseFields, PlaceholderView};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2", get(list))
        .route("/oauth2/new", get(create))
        .route("/oauth2/{id}", get(detail))
}

pub async fn list(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "oauth2"),
        section_label: "OAuth2 Apps",
        message: "Browse, create, and manage OAuth2 resource servers in your Kanidm instance.",
        phase_label: "Phase 4",
    })
}

pub async fn create(
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

pub async fn detail(
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

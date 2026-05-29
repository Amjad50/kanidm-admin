use axum::extract::State;
use axum::routing::get;
use axum::Router;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::views::{BaseFields, PlaceholderView};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/people", get(list))
        .route("/people/new", get(create))
        .route("/people/{id}", get(detail))
}

pub async fn list(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "people"),
        section_label: "People",
        message: "Browse, create, and manage user accounts in your Kanidm instance.",
        phase_label: "Phase 2",
    })
}

pub async fn create(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "people"),
        section_label: "New Person",
        message: "Create a new user account in your Kanidm instance.",
        phase_label: "Phase 2",
    })
}

pub async fn detail(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "people"),
        section_label: "Person Detail",
        message: "View and edit a user account in your Kanidm instance.",
        phase_label: "Phase 2",
    })
}

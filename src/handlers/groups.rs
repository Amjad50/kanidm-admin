use axum::extract::State;
use axum::routing::get;
use axum::Router;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::views::{BaseFields, PlaceholderView};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/groups", get(list))
        .route("/groups/new", get(create))
        .route("/groups/{id}", get(detail))
}

pub async fn list(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "groups"),
        section_label: "Groups",
        message: "Browse, create, and manage groups in your Kanidm instance.",
        phase_label: "Phase 3",
    })
}

pub async fn create(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "groups"),
        section_label: "New Group",
        message: "Create a new group in your Kanidm instance.",
        phase_label: "Phase 3",
    })
}

pub async fn detail(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "groups"),
        section_label: "Group Detail",
        message: "View and edit a group in your Kanidm instance.",
        phase_label: "Phase 3",
    })
}

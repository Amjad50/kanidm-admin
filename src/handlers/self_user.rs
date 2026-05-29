use axum::extract::State;
use axum::routing::get;
use axum::Router;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::views::{BaseFields, PlaceholderView};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(profile))
        .route("/me/sessions", get(sessions))
}

pub async fn profile(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "me"),
        section_label: "My Profile",
        message: "View and update your account details and credentials.",
        phase_label: "Phase 5",
    })
}

pub async fn sessions(
    State(_state): State<AppState>,
    user: AdminUser,
) -> AppResult<PlaceholderView> {
    Ok(PlaceholderView {
        base: BaseFields::new(&user, "me"),
        section_label: "My Sessions",
        message: "Review and revoke your active sessions across all devices.",
        phase_label: "Phase 5",
    })
}

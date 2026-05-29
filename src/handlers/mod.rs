mod dashboard;
mod health;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/", get(dashboard::dashboard))
        .route("/admin", get(dashboard::dashboard))
}

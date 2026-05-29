mod auth;
mod config;
mod error;
mod handlers;
mod kanidm;
mod views;

use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::auth::{KanidmClientFactory, PendingAuthStore};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub kanidm: Arc<KanidmClientFactory>,
    pub pending: Arc<PendingAuthStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,kanidm_admin_ui=debug")),
        )
        .init();

    let config = Arc::new(Config::load().context("loading config")?);
    let kanidm =
        Arc::new(KanidmClientFactory::new(&config).context("building kanidm client factory")?);

    let state = AppState {
        config: config.clone(),
        kanidm,
        pending: Arc::new(PendingAuthStore::new()),
    };

    let app = Router::new()
        .merge(handlers::router())
        .nest_service("/admin/static", ServeDir::new(&config.static_dir))
        .fallback(handlers::not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("binding to {}", config.bind_addr))?;

    tracing::info!("kanidm-admin-ui listening on http://{}", config.bind_addr);

    axum::serve(listener, app).await.context("axum serve")?;
    Ok(())
}

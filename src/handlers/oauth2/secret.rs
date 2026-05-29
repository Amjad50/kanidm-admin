use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::views::partials::Modal;
use crate::AppState;

use crate::handlers::common::friendly_client_error;

use super::common::OAuth2Kind;
use super::detail::{compute_header, fetch_oauth2_entry, OAuth2Header};

const KEY_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/></svg>"#;

// ── Data model ────────────────────────────────────────────────────────────────

pub enum SecretData {
    Basic {
        /// The current secret value, if one is set.
        secret: Option<String>,
        /// `true` immediately after a regenerate — auto-reveals the secret.
        fresh: bool,
        /// Friendly error message from a failed regenerate.
        error: Option<String>,
    },
    Public,
}

// ── Modal footer fragment ─────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "oauth2/_secret_modal_footer.html")]
struct SecretModalFooter {}

// ── HTMX card fragment ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "oauth2/_secret_card.html")]
pub struct SecretCardFragment {
    pub header: OAuth2Header,
    pub data: SecretData,
}

impl IntoResponse for SecretCardFragment {
    fn into_response(self) -> Response {
        match askama::Template::render(&self) {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::Template(e).into_response(),
        }
    }
}

// ── GET /oauth2/{id}/secret ───────────────────────────────────────────────────

pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    // Non-HTMX: redirect back to the overview tab (full-page secret is gone).
    if !is_htmx {
        return Ok(Redirect::to(&format!("/oauth2/{id}/overview")).into_response());
    }

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);

    // Compute modal metadata before header is moved into the body fragment.
    let (modal_title, icon_color_class) = match &header.kind {
        OAuth2Kind::Basic => (
            format!("{} — Basic secret", header.displayname),
            "text-accent",
        ),
        OAuth2Kind::Public => (
            format!("{} — Client type", header.displayname),
            "text-info",
        ),
    };

    let data = match &header.kind {
        OAuth2Kind::Public => SecretData::Public,
        OAuth2Kind::Basic => {
            let client = state
                .kanidm
                .for_token(&user.token)
                .await
                .map_err(|e| AppError::Kanidm(e.to_string()))?;

            let (secret, error) = match client.idm_oauth2_rs_get_basic_secret(&id).await {
                Ok(opt) => (opt, None),
                Err(e) => {
                    tracing::warn!(id, error = ?e, "fetching oauth2 basic secret failed");
                    let msg = friendly_client_error("fetch oauth2 secret", &e);
                    (None, Some(msg))
                }
            };

            SecretData::Basic { secret, fresh: false, error }
        }
    };

    let body_html = SecretCardFragment { header, data }
        .render()
        .map_err(AppError::Template)?;

    let footer_html = SecretModalFooter {}
        .render()
        .map_err(AppError::Template)?;

    let html = Modal {
        title: modal_title,
        icon_svg: Some(KEY_SVG),
        icon_color_class,
        body_html,
        footer_html,
        size_class: "max-w-lg",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

// ── POST /oauth2/{id}/secret/reset ───────────────────────────────────────────

pub async fn reset(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    // Trigger regeneration by passing reset_secret=true (clears the attribute,
    // causing kanidm to generate a new one on the next read).
    let reset_error = match client.idm_oauth2_rs_update(&id, None, None, None, true).await {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(id, error = ?e, "oauth2 secret reset failed");
            Some(friendly_client_error("reset oauth2 secret", &e))
        }
    };

    // Always re-fetch the entry for the header, then read the (possibly new) secret.
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);

    let data = if let Some(err) = reset_error {
        // Reset failed — show whatever secret is currently set with the error.
        let current = match client.idm_oauth2_rs_get_basic_secret(&id).await {
            Ok(opt) => opt,
            Err(get_err) => {
                tracing::warn!(id, error = ?get_err, "fetching oauth2 secret after failed reset");
                None
            }
        };
        SecretData::Basic { secret: current, fresh: false, error: Some(err) }
    } else {
        // Reset succeeded — read the new secret and auto-reveal it.
        let new_secret = match client.idm_oauth2_rs_get_basic_secret(&id).await {
            Ok(opt) => opt,
            Err(e) => {
                tracing::warn!(id, error = ?e, "fetching new oauth2 secret after reset");
                None
            }
        };
        SecretData::Basic { secret: new_secret, fresh: true, error: None }
    };

    Ok(SecretCardFragment { header, data }.into_response())
}

use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::views::BaseFields;
use crate::AppState;

use crate::handlers::common::friendly_client_error;

use super::common::OAuth2Kind;
use super::detail::{compute_header, fetch_oauth2_entry, OAuth2Header};

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

// ── Full-page view ────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "oauth2/secret.html")]
pub struct SecretView {
    pub base: BaseFields,
    pub header: OAuth2Header,
    pub data: SecretData,
}

impl IntoResponse for SecretView {
    fn into_response(self) -> Response {
        match askama::Template::render(&self) {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::Template(e).into_response(),
        }
    }
}

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
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);

    let data = match header.kind {
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

    if is_htmx {
        return Ok(SecretCardFragment { header, data }.into_response());
    }

    Ok(SecretView { base: BaseFields::new(&user, "oauth2"), header, data }.into_response())
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

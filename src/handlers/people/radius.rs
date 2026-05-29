use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::AppState;

use super::common::friendly_client_error;
use super::detail::{compute_header, fetch_person, render_detail, TabContent};

// ── View data ─────────────────────────────────────────────────────────────────

pub struct RadiusData {
    pub person_id: String,
    /// `Some(secret)` when a RADIUS credential is configured; `None` otherwise.
    pub secret: Option<String>,
    /// Friendly error message to surface in the UI (e.g. on regenerate failure).
    pub error: Option<String>,
}

// ── GET /people/{id}/radius ───────────────────────────────────────────────────

pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let secret = match client.idm_account_radius_credential_get(&id).await {
        Ok(opt) => opt,
        Err(e) => {
            tracing::warn!(error = ?e, id, "fetching radius secret failed");
            None
        }
    };

    let tab_content = TabContent::Radius(RadiusData {
        person_id: id,
        secret,
        error: None,
    });

    render_detail(is_htmx, user, person, "radius", tab_content)
}

// ── POST /people/{id}/radius/regenerate ───────────────────────────────────────

pub async fn regenerate(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let (secret, error) = match client.idm_account_radius_credential_regenerate(&id).await {
        Ok(new_secret) => (Some(new_secret), None),
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "RADIUS credential regenerate failed");
            let err_msg = friendly_client_error("regenerate RADIUS secret", &e);
            // Fall back to reading the current secret so we can still display it.
            let current = match client.idm_account_radius_credential_get(&id).await {
                Ok(opt) => opt,
                Err(get_err) => {
                    tracing::warn!(error = ?get_err, id, "fetching radius secret after failed regenerate");
                    None
                }
            };
            (current, Some(err_msg))
        }
    };

    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);

    let tab_content = TabContent::Radius(RadiusData {
        person_id: id,
        secret,
        error,
    });

    render_radius_fragment(person, tab_content)
}

// ── POST /people/{id}/radius/delete ───────────────────────────────────────────

pub async fn delete_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    if let Err(e) = client.idm_account_radius_credential_delete(&id).await {
        tracing::warn!(person = %id, error = ?e, "RADIUS credential delete failed");
    }

    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);

    let tab_content = TabContent::Radius(RadiusData {
        person_id: id,
        secret: None,
        error: None,
    });

    render_radius_fragment(person, tab_content)
}

// ── Fragment renderer (POST responses return only #tab-content inner) ─────────

fn render_radius_fragment(
    person: super::detail::PersonHeader,
    tab_content: TabContent,
) -> AppResult<Response> {
    use super::detail::TabContentFragment;

    let html = askama::Template::render(&TabContentFragment {
        tab_content: &tab_content,
        person: &person,
    })
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

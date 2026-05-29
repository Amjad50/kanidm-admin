use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::attr_first;
use crate::views::initials;
use crate::views::partials::{DeleteFooter, DestructiveConfirm, IdentityRow, Modal};
use crate::AppState;

use super::common::friendly_client_error;
use crate::handlers::common::safe_id;
use super::detail::fetch_person;

/// Build the complete delete modal HTML.  Called by both GET and POST (error
/// path) so all rendering logic lives in one place.
async fn build_modal(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    error: Option<String>,
) -> AppResult<String> {
    let entry = fetch_person(state, user, id).await?;

    let spn = attr_first(&entry, "spn").unwrap_or_else(|| id.to_string());
    let displayname = attr_first(&entry, "displayname")
        .or_else(|| attr_first(&entry, "name"))
        .unwrap_or_else(|| spn.clone());

    let input_id = format!("people-delete-{}", safe_id(id));

    let target_html = IdentityRow {
        initials: initials(&displayname),
        displayname: displayname.clone(),
        spn: spn.clone(),
    }
    .render()
    .map_err(AppError::Template)?;

    let confirm_token_js = serde_json::to_string(&spn).unwrap_or_else(|_| format!("{:?}", spn));

    let body_html = DestructiveConfirm {
        lead_text: "You're about to delete:".to_string(),
        target_html,
        consequences: vec![
            format!("{displayname} will be signed out from all devices immediately."),
            "Their account moves to the recycle bin and is recoverable for a limited time."
                .to_string(),
            "Active OAuth2 tokens they have signed are revoked.".to_string(),
        ],
        confirm_token: spn.clone(),
        confirm_token_js,
        confirm_label: "Type the SPN to confirm:".to_string(),
        input_id: input_id.clone(),
        error,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = DeleteFooter {
        action_url: format!("/people/{id}/delete"),
        confirm_label: "Delete person".to_string(),
        input_id,
        hx_vals_json: None,
    }
    .render()
    .map_err(AppError::Template)?;

    Modal {
        title: "Delete person".to_string(),
        icon_name: Some("shield-alert"),
        icon_color_class: "text-danger",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)
}

/// GET /people/{id}/delete — returns the delete modal HTML for HTMX overlay swap.
pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let html = build_modal(&state, &user, &id, None).await?;
    Ok(Html(html).into_response())
}

/// POST /people/{id}/delete — executes the delete.
/// On success: returns `HX-Redirect: /people` so HTMX navigates the browser.
/// On error: re-renders the modal with an error banner.
pub async fn submit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client.idm_person_account_delete(&id).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Redirect",
                "/people".parse().expect("static header value is valid"),
            );
            Ok((headers, Html(String::new())).into_response())
        }
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "delete person failed");
            let msg = friendly_client_error("delete person", &e);
            let html = build_modal(&state, &user, &id, Some(msg)).await?;
            Ok(Html(html).into_response())
        }
    }
}

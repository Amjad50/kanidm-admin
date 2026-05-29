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
use super::detail::fetch_person;

/// The shield-warning SVG used as the modal icon.
const SHIELD_WARNING_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>"#;

/// Sanitise an arbitrary string to a safe HTML id: strip whitespace, replace
/// characters that would break a CSS selector or `getElementById` call.
fn safe_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect()
}

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
    }
    .render()
    .map_err(AppError::Template)?;

    Modal {
        title: "Delete person".to_string(),
        icon_svg: Some(SHIELD_WARNING_SVG),
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

use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::safe_id;
use crate::kanidm::entry::attr_first;
use crate::views::partials::{DeleteFooter, DestructiveConfirm, IdentityRow, Modal};
use crate::AppState;

use super::common::friendly_error;

async fn build_modal(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    error: Option<String>,
) -> AppResult<String> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let entry = client
        .idm_group_get(id)
        .await
        .map_err(|e| AppError::Kanidm(format!("group get failed: {e:?}")))?
        .ok_or(AppError::NotFound)?;

    let spn = attr_first(&entry, "spn").unwrap_or_else(|| id.to_string());
    let name = attr_first(&entry, "name").unwrap_or_else(|| spn.clone());

    let input_id = format!("group-delete-{}", safe_id(id));

    // Use group name as "initials" since groups have no avatar — show first two chars
    let initials: String = name.chars().take(2).collect::<String>().to_uppercase();

    let target_html = IdentityRow {
        initials,
        displayname: name.clone(),
        spn: spn.clone(),
    }
    .render()
    .map_err(AppError::Template)?;

    let body_html = DestructiveConfirm {
        lead_text: "You're about to delete:".to_string(),
        target_html,
        consequences: vec![
            "All group memberships will be removed immediately.".to_string(),
            "The group moves to the recycle bin and is recoverable for a limited time.".to_string(),
            "OAuth2 scope maps referencing this group will stop granting scopes.".to_string(),
        ],
        confirm_token: name.clone(),
        confirm_label: "Type the group name to confirm:".to_string(),
        input_id: input_id.clone(),
        error,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = DeleteFooter {
        action_url: format!("/groups/{id}/delete"),
        confirm_label: "Delete group".to_string(),
        input_id,
        hx_vals_json: None,
    }
    .render()
    .map_err(AppError::Template)?;

    Modal {
        title: "Delete group".to_string(),
        icon_name: Some("shield-alert"),
        icon_color_class: "text-danger",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)
}

/// GET /groups/{id}/delete
pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let html = build_modal(&state, &user, &id, None).await?;
    Ok(Html(html).into_response())
}

/// POST /groups/{id}/delete
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

    match client.idm_group_delete(&id).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Redirect",
                "/groups".parse().expect("static header value is valid"),
            );
            Ok((headers, Html(String::new())).into_response())
        }
        Err(e) => {
            tracing::warn!(group = %id, error = ?e, "delete group failed");
            let msg = friendly_error("delete group", &e);
            let html = build_modal(&state, &user, &id, Some(msg)).await?;
            Ok(Html(html).into_response())
        }
    }
}

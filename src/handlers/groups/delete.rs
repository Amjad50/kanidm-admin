use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::attr_first;
use crate::views::partials::{DeleteFooter, DestructiveConfirm, IdentityRow, Modal};
use crate::AppState;

use super::common::friendly_error;

const SHIELD_WARNING_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>"#;

fn safe_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect()
}

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

    let confirm_token_js = serde_json::to_string(&name).unwrap_or_else(|_| format!("{:?}", name));

    let body_html = DestructiveConfirm {
        lead_text: "You're about to delete:".to_string(),
        target_html,
        consequences: vec![
            "All group memberships will be removed immediately.".to_string(),
            "The group moves to the recycle bin and is recoverable for a limited time.".to_string(),
            "OAuth2 scope maps referencing this group will stop granting scopes.".to_string(),
        ],
        confirm_token: name.clone(),
        confirm_token_js,
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
    }
    .render()
    .map_err(AppError::Template)?;

    Modal {
        title: "Delete group".to_string(),
        icon_svg: Some(SHIELD_WARNING_SVG),
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

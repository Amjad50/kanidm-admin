use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::{friendly_client_error, safe_id};
use crate::kanidm::entry::attr_first;
use crate::views::partials::{DeleteFooter, DestructiveConfirm, IdentityRow, Modal};
use crate::AppState;


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
        .idm_oauth2_rs_get(id)
        .await
        .map_err(|e| AppError::Kanidm(format!("oauth2 get failed: {e:?}")))?
        .ok_or(AppError::NotFound)?;

    let spn = attr_first(&entry, "spn").unwrap_or_else(|| id.to_string());
    let name = attr_first(&entry, "name").unwrap_or_else(|| spn.clone());

    let input_id = format!("oauth2-delete-{}", safe_id(id));

    // Use client name as "initials" — show first two chars uppercased
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
            "Tokens issued by this client will continue to validate until they expire; the client cannot mint new ones.".to_string(),
            "Any application integrated with this client will lose SSO until reconfigured.".to_string(),
            "Scope and claim maps configured for this client are removed with the entry.".to_string(),
        ],
        confirm_token: name.clone(),
        confirm_token_js,
        confirm_label: "Type the OAuth2 client name to confirm:".to_string(),
        input_id: input_id.clone(),
        error,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = DeleteFooter {
        action_url: format!("/oauth2/{id}/delete"),
        confirm_label: "Delete OAuth2 client".to_string(),
        input_id,
        hx_vals_json: None,
    }
    .render()
    .map_err(AppError::Template)?;

    Modal {
        title: "Delete OAuth2 client".to_string(),
        icon_name: Some("shield-alert"),
        icon_color_class: "text-danger",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)
}

/// GET /oauth2/{id}/delete
pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let html = build_modal(&state, &user, &id, None).await?;
    Ok(Html(html).into_response())
}

/// POST /oauth2/{id}/delete
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

    match client.idm_oauth2_rs_delete(&id).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", "/oauth2".parse().expect("valid header"));
            Ok((headers, Html(String::new())).into_response())
        }
        Err(e) => {
            tracing::warn!(oauth2 = %id, error = ?e, "delete oauth2 client failed");
            let msg = friendly_client_error("delete oauth2 client", &e);
            let html = build_modal(&state, &user, &id, Some(msg)).await?;
            Ok(Html(html).into_response())
        }
    }
}

use askama::Template;
use axum::extract::{Form, Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::views::partials::{Modal, OneTimeSecret};
use crate::views::{format_absolute, format_relative_future};
use crate::AppState;

use super::common::{friendly_client_error, summarize_credentials};
use super::detail::{compute_header, fetch_person, render_detail, TabContent};

// ── View data structs ─────────────────────────────────────────────────────────

pub struct PrimaryCredentialInfo {
    pub label: String,
    pub totp_labels: Vec<String>,
    pub backup_code_count: usize,
}

pub struct PasskeyInfo {
    pub count: usize,
    pub names: Vec<String>,
}

pub struct CredentialsData {
    pub person_id: String,
    pub primary: Option<PrimaryCredentialInfo>,
    pub passkeys: PasskeyInfo,
    pub attested_passkeys: PasskeyInfo,
    pub ssh_key_count: usize,
    pub radius_configured: bool,
    pub cred_status_error: Option<String>,
}

// ── Form data ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct ResetForm {
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

fn default_ttl() -> u32 {
    1
}

fn ttl_to_seconds(ttl: u32) -> u32 {
    match ttl {
        1 => 3600,
        8 => 28800,
        24 => 86400,
        168 => 604800,
        _ => {
            tracing::warn!(ttl, "unexpected ttl value; defaulting to 1 hour");
            3600
        }
    }
}

// ── QR code helper ────────────────────────────────────────────────────────────

fn build_qr_svg(url: &str) -> Option<String> {
    use qrcode::render::svg;
    use qrcode::QrCode;

    let code = QrCode::new(url.as_bytes()).ok()?;
    let svg_string = code
        .render::<svg::Color>()
        .min_dimensions(160, 160)
        .max_dimensions(200, 200)
        .dark_color(svg::Color("#1a1a1f"))
        .light_color(svg::Color("#f4f4f7"))
        .build();
    Some(svg_string)
}

// ── Tab GET handler ───────────────────────────────────────────────────────────

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

    let cred_status_result = client.idm_person_account_get_credential_status(&id).await;

    let (cred_summary, cred_status_error) = match cred_status_result {
        Ok(status) => (summarize_credentials(&entry, Some(&status)), None),
        Err(kanidm_client::ClientError::EmptyResponse) => {
            (summarize_credentials(&entry, None), None)
        }
        Err(e) => {
            tracing::warn!(?id, error = ?e, "credential status fetch failed");
            (
                summarize_credentials(&entry, None),
                Some("Could not load credential details.".to_string()),
            )
        }
    };

    let primary = cred_summary.primary.label().map(|label| PrimaryCredentialInfo {
        label: label.to_string(),
        totp_labels: cred_summary.totp_labels,
        backup_code_count: cred_summary.backup_codes_remaining.unwrap_or(0),
    });
    let passkeys = PasskeyInfo {
        count: cred_summary.passkey_count,
        names: cred_summary.passkey_names,
    };
    let attested_passkeys = PasskeyInfo {
        count: cred_summary.attested_passkey_count,
        names: cred_summary.attested_passkey_names,
    };

    let tab_content = TabContent::Credentials(CredentialsData {
        person_id: id.clone(),
        primary,
        passkeys,
        attested_passkeys,
        ssh_key_count: cred_summary.ssh_key_count,
        radius_configured: cred_summary.radius_configured,
        cred_status_error,
    });

    render_detail(is_htmx, user, person, "credentials", tab_content)
}

// ── Reset modal GET handler ───────────────────────────────────────────────────

pub async fn reset_modal(Path(id): Path<String>, _user: AdminUser) -> AppResult<Response> {
    let body_html = ResetModalBody { person_id: id.clone() }
        .render()
        .map_err(AppError::Template)?;

    let footer_html = ResetModalFooter {}
        .render()
        .map_err(AppError::Template)?;

    let html = Modal {
        title: "Generate reset link".to_string(),
        icon_name: Some("key"),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-sm",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}


#[derive(Template)]
#[template(path = "people/_credentials_reset_modal.html")]
pub struct ResetModalBody {
    pub person_id: String,
}

#[derive(Template)]
#[template(path = "people/_credentials_reset_modal_footer.html")]
pub struct ResetModalFooter {}

// ── Reset POST handler ────────────────────────────────────────────────────────

pub async fn submit_reset(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<ResetForm>,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let ttl_secs = ttl_to_seconds(form.ttl);

    match client
        .idm_person_account_credential_update_intent(&id, Some(ttl_secs))
        .await
    {
        Ok(token) => {
            let reset_url =
                format!("{}/ui/reset?token={}", state.config.kanidm_url, token.token);
            let qr_svg = build_qr_svg(&reset_url);

            let relative = format_relative_future(token.expiry_time);
            let absolute = format_absolute(token.expiry_time);

            let body_html = OneTimeSecret {
                label: "Reset URL".to_string(),
                value: reset_url,
                helper: Some(
                    "Share this link with the account holder. They'll use it to set new credentials. \
                     Re-generating creates a new link and invalidates this one."
                        .to_string(),
                ),
                copy_aria: "Copy reset URL".to_string(),
                expires_relative: Some(relative),
                expires_absolute: Some(absolute),
                qr_svg,
                action_html: None,
            }
            .render()
            .map_err(AppError::Template)?;

            let footer_html = ResetResultFooter { person_id: id.clone() }
                .render()
                .map_err(AppError::Template)?;

            let modal_html = Modal {
                title: "Reset link ready".to_string(),
                icon_name: Some("circle-check"),
                icon_color_class: "text-success",
                body_html,
                footer_html,
                size_class: "max-w-xl",
            }
            .render()
            .map_err(AppError::Template)?;

            Ok(Html(modal_html).into_response())
        }
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "credential reset intent failed");
            let msg = friendly_client_error("generate reset link", &e);

            let body_html = ResetErrorBody { message: msg }
                .render()
                .map_err(AppError::Template)?;

            let footer_html = ResetResultFooter { person_id: id.clone() }
                .render()
                .map_err(AppError::Template)?;

            let modal_html = Modal {
                title: "Reset link failed".to_string(),
                icon_name: Some("circle-x"),
                icon_color_class: "text-danger",
                body_html,
                footer_html,
                size_class: "max-w-sm",
            }
            .render()
            .map_err(AppError::Template)?;

            Ok(Html(modal_html).into_response())
        }
    }
}



#[derive(Template)]
#[template(path = "people/_credentials_reset_result_footer.html")]
pub struct ResetResultFooter {
    pub person_id: String,
}

#[derive(Template)]
#[template(path = "people/_credentials_reset_error_body.html")]
pub struct ResetErrorBody {
    pub message: String,
}


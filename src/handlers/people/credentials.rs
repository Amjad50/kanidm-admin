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
    pub primary: Option<PrimaryCredentialInfo>,
    pub passkeys: PasskeyInfo,
    pub attested_passkeys: PasskeyInfo,
    pub ssh_key_count: usize,
    pub radius_configured: bool,
    pub cred_status_error: Option<String>,
    /// Pre-rendered reset card HTML (via `ResetCard::render()`).
    pub reset_card_html: String,
}

pub struct ResetLinkResult {
    pub secret_html: String,
    pub error: Option<String>,
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

// ── Regenerate button ─────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "people/_credentials_regenerate_button.html")]
pub struct RegenerateButton {
    pub person_id: String,
}

// ── Reset card builder ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "people/_credentials_reset.html")]
pub struct ResetCard {
    pub person_id: String,
    pub reset_result: Option<ResetLinkResult>,
}

fn build_reset_card(person_id: &str, reset_result: Option<ResetLinkResult>) -> AppResult<String> {
    ResetCard {
        person_id: person_id.to_string(),
        reset_result,
    }
    .render()
    .map_err(AppError::Template)
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

    let reset_card_html = build_reset_card(&id, None)?;

    let tab_content = TabContent::Credentials(CredentialsData {
        primary,
        passkeys,
        attested_passkeys,
        ssh_key_count: cred_summary.ssh_key_count,
        radius_configured: cred_summary.radius_configured,
        cred_status_error,
        reset_card_html,
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
        icon_svg: Some(KEY_SVG),
        icon_color_class: "text-accent",
        body_html,
        footer_html,
        size_class: "max-w-sm",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

const KEY_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/></svg>"#;

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

            let regenerate_html = RegenerateButton { person_id: id.clone() }
                .render()
                .map_err(AppError::Template)?;

            let secret_html = OneTimeSecret {
                label: "Reset URL".to_string(),
                value: reset_url,
                helper: Some(
                    "Shown once. Re-generating creates a new link and invalidates this one."
                        .to_string(),
                ),
                copy_aria: "Copy reset URL".to_string(),
                expires_relative: Some(relative),
                expires_absolute: Some(absolute),
                qr_svg,
                action_html: Some(regenerate_html),
            }
            .render()
            .map_err(AppError::Template)?;

            let card_html = build_reset_card(
                &id,
                Some(ResetLinkResult {
                    secret_html,
                    error: None,
                }),
            )?;

            Ok(Html(card_html).into_response())
        }
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "credential reset intent failed");
            let msg = friendly_client_error("generate reset link", &e);

            let card_html = build_reset_card(
                &id,
                Some(ResetLinkResult {
                    secret_html: String::new(),
                    error: Some(msg),
                }),
            )?;

            Ok(Html(card_html).into_response())
        }
    }
}


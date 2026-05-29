use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::Form;
use axum_htmx::HxRequest;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::attr_all;
use crate::kanidm::key_state::{parse_key_state, KeyStatus};
use crate::views::partials::{DeleteFooter, DestructiveConfirm, IdentityRow, Modal};
use crate::AppState;

use super::detail::{compute_header, fetch_oauth2_entry, render_detail, TabContent};
use crate::handlers::common::{friendly_client_error, safe_id};

// ── Data structs ──────────────────────────────────────────────────────────────

pub struct KeyRow {
    pub id: String,
    pub status_label: &'static str,
    pub status_badge_classes: &'static str,
    pub algorithm: String,
    pub counter: u64,
}

pub struct CryptoData {
    pub oauth2_id: String,
    pub keys: Vec<KeyRow>,
    pub error: Option<String>,
}

// ── Form structs ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct RotateForm {
    /// Either `"now"` or an RFC3339 / HTML `datetime-local` value.
    pub at: String,
}

#[derive(serde::Deserialize)]
pub struct RevokeForm {
    pub key_id: String,
}

#[derive(serde::Deserialize)]
pub struct RevokeModeQuery {
    pub key_id: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse the `at` field from the rotate form into an `OffsetDateTime`.
///
/// Handles three cases:
/// - `"now"` → current UTC time.
/// - RFC3339 string → parse directly.
/// - HTML `datetime-local` (`YYYY-MM-DDTHH:MM` or `YYYY-MM-DDTHH:MM:SS`) →
///   treat as UTC.
fn parse_rotate_at(s: &str) -> Result<OffsetDateTime, String> {
    let s = s.trim();

    if s == "now" || s.is_empty() {
        return Ok(OffsetDateTime::now_utc());
    }

    // Try RFC3339 first.
    if let Ok(dt) = OffsetDateTime::parse(s, &Rfc3339) {
        return Ok(dt);
    }

    // Try HTML datetime-local formats: YYYY-MM-DDTHH:MM or YYYY-MM-DDTHH:MM:SS
    // Append 'Z' to make it UTC RFC3339-parseable.
    let with_seconds = if s.len() == 16 {
        // YYYY-MM-DDTHH:MM  → add :00Z
        format!("{s}:00Z")
    } else if s.len() == 19 {
        // YYYY-MM-DDTHH:MM:SS → add Z
        format!("{s}Z")
    } else {
        return Err(format!("Could not parse rotation time: {s}"));
    };

    OffsetDateTime::parse(&with_seconds, &Rfc3339)
        .map_err(|_| format!("Could not parse rotation time: {s}"))
}

/// Build `CryptoData` from an OAuth2 entry.
fn build_crypto_data(id: &str, entry: &kanidm_proto::v1::Entry, error: Option<String>) -> CryptoData {
    let raw_keys = attr_all(entry, "key_internal_data");

    // Collect (KeyStatus, KeyRow) pairs so we can sort by status order.
    let mut pairs: Vec<(KeyStatus, KeyRow)> = raw_keys
        .iter()
        .filter_map(|v| {
            match parse_key_state(v) {
                Some(k) => {
                    let status = k.status;
                    Some((status, KeyRow {
                        id: k.id,
                        status_label: k.status.label(),
                        status_badge_classes: k.status.badge_classes(),
                        algorithm: k.algorithm,
                        counter: k.counter,
                    }))
                }
                None => {
                    tracing::warn!(
                        oauth2_id = %id,
                        raw_value = %v,
                        "key_internal_data value failed to parse — skipping"
                    );
                    None
                }
            }
        })
        .collect();

    // Sort: Valid < Retired < Revoked < Unknown; secondary by algorithm asc.
    pairs.sort_by(|(sa, ra), (sb, rb)| {
        sa.sort_order().cmp(&sb.sort_order()).then_with(|| ra.algorithm.cmp(&rb.algorithm))
    });

    let keys = pairs.into_iter().map(|(_, row)| row).collect();
    CryptoData { oauth2_id: id.to_string(), keys, error }
}

const SHIELD_WARNING_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>"#;

/// Render the revoke-key modal (shared by both GET and error re-render).
async fn build_revoke_modal(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    key_id: &str,
    error: Option<String>,
) -> AppResult<String> {
    // Verify the OAuth2 entry still exists (gives us name for the header).
    let _entry = fetch_oauth2_entry(state, user, id).await?;

    let input_id = format!("revoke-key-{}", safe_id(key_id));
    let confirm_token = key_id.to_string();
    let confirm_token_js = serde_json::to_string(&confirm_token)
        .unwrap_or_else(|_| format!("{:?}", confirm_token));

    let target_html = IdentityRow {
        initials: "KY".to_string(),
        displayname: format!("Key {key_id}"),
        spn: key_id.to_string(),
    }
    .render()
    .map_err(AppError::Template)?;

    let body_html = DestructiveConfirm {
        lead_text: "You're about to permanently revoke:".to_string(),
        target_html,
        consequences: vec![
            "The key will no longer be used to sign or verify tokens.".to_string(),
            "Tokens signed by this key will immediately be invalid.".to_string(),
            "This action cannot be undone.".to_string(),
        ],
        confirm_token: confirm_token.clone(),
        confirm_token_js,
        confirm_label: "Type the key ID to confirm:".to_string(),
        input_id: input_id.clone(),
        error,
    }
    .render()
    .map_err(AppError::Template)?;

    let footer_html = DeleteFooter {
        action_url: format!("/oauth2/{id}/crypto/revoke"),
        confirm_label: "Revoke key".to_string(),
        input_id: input_id.clone(),
        hx_vals_json: None,
    }
    .with_hx_vals(serde_json::json!({ "key_id": key_id }))
    .render()
    .map_err(AppError::Template)?;

    Modal {
        title: "Revoke signing key".to_string(),
        icon_svg: Some(SHIELD_WARNING_SVG),
        icon_color_class: "text-danger",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/crypto
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_crypto_data(&id, &entry, None);
    render_detail(is_htmx, user, header, "crypto", TabContent::Crypto(data))
}

/// POST /oauth2/{id}/crypto/rotate
pub async fn rotate(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<RotateForm>,
) -> AppResult<Response> {
    let rotate_at = match parse_rotate_at(&form.at) {
        Ok(dt) => dt,
        Err(msg) => {
            let entry = fetch_oauth2_entry(&state, &user, &id).await?;
            let header = compute_header(&state, &entry);
            let data = build_crypto_data(&id, &entry, Some(msg));
            return render_detail(is_htmx, user, header, "crypto", TabContent::Crypto(data));
        }
    };

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let error = match client.idm_oauth2_rs_rotate_keys(&id, rotate_at).await {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(oauth2_id = %id, error = ?e, "rotate keys failed");
            Some(friendly_client_error("rotate keys", &e))
        }
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_crypto_data(&id, &entry, error);
    render_detail(is_htmx, user, header, "crypto", TabContent::Crypto(data))
}

/// POST /oauth2/{id}/crypto/revoke
pub async fn revoke(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<RevokeForm>,
) -> AppResult<Response> {
    if form.key_id.is_empty() {
        let html = build_revoke_modal(
            &state,
            &user,
            &id,
            &form.key_id,
            Some("Key ID is missing.".to_string()),
        )
        .await?;
        return Ok(Html(html).into_response());
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client.idm_oauth2_rs_revoke_key(&id, &form.key_id).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Redirect",
                format!("/oauth2/{id}/crypto")
                    .parse()
                    .expect("static header value is valid"),
            );
            Ok((headers, Html(String::new())).into_response())
        }
        Err(e) => {
            tracing::warn!(oauth2_id = %id, key_id = %form.key_id, error = ?e, "revoke key failed");
            let msg = friendly_client_error("revoke key", &e);
            let html = build_revoke_modal(&state, &user, &id, &form.key_id, Some(msg)).await?;
            Ok(Html(html).into_response())
        }
    }
}

/// GET /oauth2/{id}/crypto/revoke-modal?key_id=<key_id>
pub async fn revoke_modal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<RevokeModeQuery>,
    user: AdminUser,
) -> AppResult<Response> {
    let html = build_revoke_modal(&state, &user, &id, &q.key_id, None).await?;
    Ok(Html(html).into_response())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::parse_rotate_at;
    use time::OffsetDateTime;

    #[test]
    fn parse_now() {
        let before = OffsetDateTime::now_utc();
        let dt = parse_rotate_at("now").unwrap();
        let after = OffsetDateTime::now_utc();
        assert!(dt >= before && dt <= after);
    }

    #[test]
    fn parse_empty_acts_as_now() {
        parse_rotate_at("").unwrap(); // should not error
    }

    #[test]
    fn parse_rfc3339() {
        let dt = parse_rotate_at("2026-06-01T00:00:00Z").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month() as u8, 6);
    }

    #[test]
    fn parse_datetime_local_no_seconds() {
        let dt = parse_rotate_at("2026-06-01T12:30").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn parse_datetime_local_with_seconds() {
        let dt = parse_rotate_at("2026-06-01T12:30:45").unwrap();
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn parse_garbage_returns_err() {
        assert!(parse_rotate_at("not-a-date").is_err());
    }
}

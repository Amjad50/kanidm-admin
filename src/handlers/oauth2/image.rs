use axum::extract::{Form, Multipart, Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_htmx::HxRequest;
use kanidm_proto::internal::{ImageType, ImageValue};
use axum::http::HeaderMap;
use std::time::Duration;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::kanidm::entry::attr_present;
use crate::AppState;

use super::detail::{compute_header, fetch_oauth2_entry, render_detail, TabContent};
use crate::handlers::common::friendly_client_error;

// ── 1 MiB cap applied in the handler (axum default body limit is 2 MiB) ─────
const MAX_IMAGE_BYTES: usize = 1024 * 1024;

const ALLOWED_TYPES: &str = "PNG, JPG, GIF, WEBP, SVG";

// ── Data struct ───────────────────────────────────────────────────────────────

pub struct ImageData {
    pub oauth2_id: String,
    pub has_image: bool,
    pub image_url: Option<String>,
    pub error: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_image_data(_state: &AppState, id: &str, entry: &kanidm_proto::v1::Entry, error: Option<String>) -> ImageData {
    let has_image = attr_present(entry, "image");
    let name = crate::kanidm::entry::attr_first(entry, "name").unwrap_or_default();
    let image_url = has_image.then(|| format!("/oauth2/{}/image-proxy", name));
    ImageData {
        oauth2_id: id.to_string(),
        has_image,
        image_url,
        error,
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/{id}/image-proxy — streams the kanidm-hosted image via bearer auth.
pub async fn proxy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| crate::error::AppError::Kanidm(e.to_string()))?;

    let url = client.make_url(&format!("/ui/images/oauth2/{id}"));

    let resp = client
        .client()
        .get(url)
        .bearer_auth(&user.token)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(id, error = ?e, "image proxy fetch failed");
            crate::error::AppError::Kanidm(format!("image proxy: {e}"))
        })?;

    let status = resp.status();
    if !status.is_success() {
        tracing::warn!(id, %status, "image proxy upstream non-2xx");
        return Ok((
            axum::http::StatusCode::from_u16(status.as_u16())
                .unwrap_or(axum::http::StatusCode::BAD_GATEWAY),
            "",
        )
            .into_response());
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = resp.bytes().await.map_err(|e| {
        tracing::warn!(id, error = ?e, "image proxy body read failed");
        crate::error::AppError::Kanidm(format!("image proxy body: {e}"))
    })?;

    let mut headers = HeaderMap::new();
    if let Ok(v) = axum::http::HeaderValue::from_str(&content_type) {
        headers.insert("content-type", v);
    }
    // Cache for a few minutes — admin UI is privileged anyway, so caching is fine.
    headers.insert("cache-control", "private, max-age=300".parse().unwrap());

    Ok((headers, bytes).into_response())
}

/// GET /oauth2/{id}/image
pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_image_data(&state, &id, &entry, None);
    render_detail(is_htmx, user, header, "image", TabContent::Image(data))
}

/// POST /oauth2/{id}/image — plain (non-HTMX) multipart upload
pub async fn upload(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    mut multipart: Multipart,
) -> AppResult<Response> {
    // ── Collect the `image` field from the multipart body ─────────────────
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type_str: Option<String> = None;

    loop {
        let field_opt = match multipart.next_field().await {
            Ok(opt) => opt,
            Err(e) => {
                tracing::warn!(oauth2_id = %id, error = ?e, "multipart parse failed");
                return render_error(&state, &user, &id, "Could not read upload (the file may be too large or malformed).").await;
            }
        };
        let Some(field) = field_opt else { break; };

        if field.name() == Some("image") {
            file_name = field.file_name().map(|s| s.to_string());
            content_type_str = field.content_type().map(|s| s.to_string());
            match field.bytes().await {
                Ok(b) => file_bytes = Some(b.to_vec()),
                Err(_) => {
                    return render_error(&state, &user, &id, "Failed to read uploaded file.").await;
                }
            }
            break;
        }
    }

    // ── Validate: file present and non-empty ──────────────────────────────
    let bytes = match file_bytes {
        Some(b) if !b.is_empty() => b,
        _ => {
            return render_error(&state, &user, &id, "No file uploaded. Please select an image.").await;
        }
    };

    // ── Validate: content type (before size — wrong format beats too large) ──
    let ct = content_type_str.as_deref().unwrap_or("application/octet-stream");
    let filetype = match ImageType::try_from_content_type(ct) {
        Ok(t) => t,
        Err(_) => {
            return render_error(
                &state, &user, &id,
                &format!(
                    "Unsupported file type ({ct}). Allowed formats: {ALLOWED_TYPES}."
                ),
            )
            .await;
        }
    };

    // ── Validate: file size ───────────────────────────────────────────────
    if bytes.len() > MAX_IMAGE_BYTES {
        return render_error(
            &state, &user, &id,
            &format!(
                "File is too large ({} KB). Maximum allowed size is 1 MB.",
                bytes.len() / 1024
            ),
        )
        .await;
    }

    // ── Build ImageValue and upload ───────────────────────────────────────
    let filename = file_name.unwrap_or_else(|| "image".to_string());
    let image_value = ImageValue::new(filename, filetype, bytes);

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| crate::error::AppError::Kanidm(e.to_string()))?;

    match client.idm_oauth2_rs_update_image(&id, image_value).await {
        Ok(()) => {
            // Redirect back so the page re-renders with the new image.
            Ok(Redirect::to(&format!("/oauth2/{id}/image")).into_response())
        }
        Err(e) => {
            tracing::warn!(oauth2_id = %id, error = ?e, "upload image failed");
            let msg = friendly_client_error("upload image", &e);
            render_error(&state, &user, &id, &msg).await
        }
    }
}

/// POST /oauth2/{id}/image/delete
pub async fn delete(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| crate::error::AppError::Kanidm(e.to_string()))?;

    let error = match client.idm_oauth2_rs_delete_image(&id).await {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(oauth2_id = %id, error = ?e, "delete image failed");
            Some(friendly_client_error("delete image", &e))
        }
    };

    let entry = fetch_oauth2_entry(&state, &user, &id).await?;
    let header = compute_header(&state, &entry);
    let data = build_image_data(&state, &id, &entry, error);
    render_detail(is_htmx, user, header, "image", TabContent::Image(data))
}

// ── Upload from URL ───────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct UploadFromUrlForm {
    #[serde(default)]
    pub url: String,
}

/// 10 second cap for the entire fetch; keeps a misbehaving upstream from
/// holding the admin's request handler hostage.
const URL_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Reject anything that isn't a normal HTTP(S) URL. Stops file://, ftp://,
/// data:, and the various URL schemes reqwest happens to accept.
fn validate_image_url(raw: &str) -> Result<url::Url, &'static str> {
    let parsed = url::Url::parse(raw.trim())
        .map_err(|_| "Could not parse the URL — make sure it starts with http:// or https://.")?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        _ => Err("Only http:// and https:// URLs are supported."),
    }
}

/// POST /oauth2/{id}/image/from-url — fetch a public image and push it to kanidm.
pub async fn upload_from_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<UploadFromUrlForm>,
) -> AppResult<Response> {
    // ── Validate the URL ─────────────────────────────────────────────────
    let parsed_url = match validate_image_url(&form.url) {
        Ok(u) => u,
        Err(msg) => return render_error(&state, &user, &id, msg).await,
    };

    // ── Fetch via kanidm's reqwest client (already configured with the
    //    project's TLS roots). The bearer token is NOT sent — we only use
    //    the request builder, not kanidm's authed helpers. ────────────────
    let kanidm_client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| crate::error::AppError::Kanidm(e.to_string()))?;

    let resp = match kanidm_client
        .client()
        .get(parsed_url.clone())
        .timeout(URL_FETCH_TIMEOUT)
        .header("user-agent", "kanidm-admin-ui/image-fetch")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(oauth2_id = %id, url = %parsed_url, error = ?e, "image url fetch failed");
            return render_error(
                &state, &user, &id,
                &format!("Could not fetch the URL: {e}"),
            )
            .await;
        }
    };

    if !resp.status().is_success() {
        tracing::warn!(oauth2_id = %id, url = %parsed_url, status = %resp.status(), "image url upstream non-2xx");
        return render_error(
            &state, &user, &id,
            &format!("The URL returned HTTP {} — make sure it's a public image link.", resp.status().as_u16()),
        )
        .await;
    }

    // ── Validate content-type (before reading the body) ─────────────────
    let ct_str = resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
        .unwrap_or_default();

    let filetype = match ImageType::try_from_content_type(&ct_str) {
        Ok(t) => t,
        Err(_) => {
            return render_error(
                &state, &user, &id,
                &format!(
                    "The URL did not return an image (content-type: {}). Allowed formats: {ALLOWED_TYPES}.",
                    if ct_str.is_empty() { "missing" } else { ct_str.as_str() }
                ),
            )
            .await;
        }
    };

    // ── Reject early if Content-Length advertises something too big ─────
    if let Some(len) = resp.content_length() {
        if len as usize > MAX_IMAGE_BYTES {
            return render_error(
                &state, &user, &id,
                &format!(
                    "Image at the URL is too large ({} KB). Maximum allowed size is 1 MB.",
                    len / 1024
                ),
            )
            .await;
        }
    }

    // ── Stream the body with a size guard (don't trust Content-Length) ───
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(oauth2_id = %id, url = %parsed_url, error = ?e, "image url body read failed");
            return render_error(&state, &user, &id, "Could not read the image body from the URL.").await;
        }
    };

    if bytes.is_empty() {
        return render_error(&state, &user, &id, "The URL returned an empty body.").await;
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return render_error(
            &state, &user, &id,
            &format!(
                "Image at the URL is too large ({} KB). Maximum allowed size is 1 MB.",
                bytes.len() / 1024
            ),
        )
        .await;
    }

    // ── Derive a sensible filename from the URL path ─────────────────────
    let filename = parsed_url
        .path_segments()
        .and_then(|segs| segs.last())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "image".to_string());

    let image_value = ImageValue::new(filename, filetype, bytes.to_vec());

    // ── Push to kanidm ───────────────────────────────────────────────────
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| crate::error::AppError::Kanidm(e.to_string()))?;

    match client.idm_oauth2_rs_update_image(&id, image_value).await {
        Ok(()) => Ok(Redirect::to(&format!("/oauth2/{id}/image")).into_response()),
        Err(e) => {
            tracing::warn!(oauth2_id = %id, url = %parsed_url, error = ?e, "upload image from url failed");
            let msg = friendly_client_error("upload image", &e);
            render_error(&state, &user, &id, &msg).await
        }
    }
}

// ── Error re-render helper ────────────────────────────────────────────────────

/// Re-render the image tab (full-page, non-HTMX) with an error banner.
/// Upload is a plain POST, so we always do a full-page render here.
async fn render_error(
    state: &AppState,
    user: &AdminUser,
    id: &str,
    msg: &str,
) -> AppResult<Response> {
    let entry = fetch_oauth2_entry(state, user, id).await?;
    let header = compute_header(state, &entry);
    let data = build_image_data(state, id, &entry, Some(msg.to_string()));
    // Upload is a non-HTMX plain POST, so always full page.
    render_detail(false, user.clone(), header, "image", TabContent::Image(data))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use kanidm_proto::internal::ImageType;

    #[test]
    fn content_type_png_accepted() {
        assert!(matches!(
            ImageType::try_from_content_type("image/png"),
            Ok(ImageType::Png)
        ));
    }

    #[test]
    fn content_type_jpeg_accepted() {
        assert!(matches!(
            ImageType::try_from_content_type("image/jpeg"),
            Ok(ImageType::Jpg)
        ));
    }

    #[test]
    fn content_type_svg_accepted() {
        assert!(matches!(
            ImageType::try_from_content_type("image/svg+xml"),
            Ok(ImageType::Svg)
        ));
    }

    #[test]
    fn content_type_webp_accepted() {
        assert!(matches!(
            ImageType::try_from_content_type("image/webp"),
            Ok(ImageType::Webp)
        ));
    }

    #[test]
    fn content_type_gif_accepted() {
        assert!(matches!(
            ImageType::try_from_content_type("image/gif"),
            Ok(ImageType::Gif)
        ));
    }

    #[test]
    fn content_type_pdf_rejected() {
        assert!(ImageType::try_from_content_type("application/pdf").is_err());
    }

    #[test]
    fn content_type_octet_rejected() {
        assert!(ImageType::try_from_content_type("application/octet-stream").is_err());
    }

    use super::validate_image_url;

    #[test]
    fn url_https_accepted() {
        assert!(validate_image_url("https://example.com/logo.png").is_ok());
    }

    #[test]
    fn url_http_accepted() {
        assert!(validate_image_url("http://example.com/logo.png").is_ok());
    }

    #[test]
    fn url_file_rejected() {
        assert!(validate_image_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn url_ftp_rejected() {
        assert!(validate_image_url("ftp://example.com/x.png").is_err());
    }

    #[test]
    fn url_data_rejected() {
        assert!(validate_image_url("data:image/png;base64,iVBORw0KG...").is_err());
    }

    #[test]
    fn url_malformed_rejected() {
        assert!(validate_image_url("not a url").is_err());
    }
}

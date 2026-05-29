use axum::extract::{Form, Path, State};
use axum::response::Response;
use axum_htmx::HxRequest;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::kanidm::entry::attr_all;
use crate::kanidm::ssh::{
    parse_ssh_publickey_line, validate_pubkey_shape, validate_tag, ParsedSshKey,
};
use crate::AppState;

use super::common::friendly_client_error;
use super::create::FormField;
use super::detail::{compute_header, fetch_person, render_detail, TabContent};

// ── SSH key model ─────────────────────────────────────────────────────────────

/// View-level representation of a single SSH public key (ready for template use).
pub struct SshKey {
    pub tag: String,
    pub algorithm_friendly: String,
    pub fingerprint: String,
    pub fingerprint_truncated: String,
    pub openssh_line: String,
    pub openssh_line_preview: String,
}

impl From<ParsedSshKey> for SshKey {
    fn from(p: ParsedSshKey) -> Self {
        SshKey {
            tag: p.tag,
            algorithm_friendly: p.algorithm_friendly,
            fingerprint: p.fingerprint,
            fingerprint_truncated: p.fingerprint_truncated,
            openssh_line: p.openssh_line,
            openssh_line_preview: p.openssh_line_preview,
        }
    }
}

// ── Form data ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct AddSshForm {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub pubkey: String,
}

// ── View data ─────────────────────────────────────────────────────────────────

pub struct SshData {
    pub person_id: String,
    pub keys: Vec<SshKey>,
    pub tag_field: FormField,
    pub pubkey_field: FormField,
    pub form_error: Option<String>,
}

// ── View builder ──────────────────────────────────────────────────────────────

fn build_ssh_data(
    person_id: &str,
    keys: Vec<SshKey>,
    form: AddSshForm,
    tag_error: Option<String>,
    pubkey_error: Option<String>,
    form_error: Option<String>,
) -> SshData {
    SshData {
        person_id: person_id.to_string(),
        keys,
        tag_field: FormField {
            id: "tag",
            name: "tag",
            label: "Label",
            input_type: "text",
            value: form.tag,
            placeholder: "laptop_ed25519",
            required: true,
            autofocus: false,
            suffix: None,
            helper: Some("Short identifier for this key. Letters, digits, '.', '_', '-'."),
            error: tag_error,
            multiline: false,
            rows: 0,
        },
        pubkey_field: FormField {
            id: "pubkey",
            name: "pubkey",
            label: "Public key",
            input_type: "text",
            value: form.pubkey,
            placeholder: "ssh-ed25519 AAAA…",
            required: true,
            autofocus: false,
            suffix: None,
            helper: Some("Paste a single OpenSSH-format public key."),
            error: pubkey_error,
            multiline: true,
            rows: 4,
        },
        form_error,
    }
}

fn parse_keys_from_entry(entry: &kanidm_proto::v1::Entry) -> Vec<SshKey> {
    attr_all(entry, "ssh_publickey")
        .into_iter()
        .filter_map(|line| parse_ssh_publickey_line(&line).map(SshKey::from))
        .collect()
}

// ── GET /people/{id}/ssh ──────────────────────────────────────────────────────

pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let keys = parse_keys_from_entry(&entry);
    let ssh_data = build_ssh_data(&id, keys, AddSshForm::default(), None, None, None);
    render_detail(is_htmx, user, person, "ssh", TabContent::Ssh(ssh_data))
}

// ── POST /people/{id}/ssh ─────────────────────────────────────────────────────

pub async fn add(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<AddSshForm>,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let existing_keys = parse_keys_from_entry(&entry);

    let tag_err = validate_tag(&form.tag).err().map(str::to_owned);
    let pubkey_err = validate_pubkey_shape(&form.pubkey).err().map(str::to_owned);

    let dup_err = if tag_err.is_none() {
        let tag = form.tag.trim();
        if existing_keys.iter().any(|k| k.tag == tag) {
            Some(format!("A key with label \"{tag}\" already exists."))
        } else {
            None
        }
    } else {
        None
    };

    let effective_tag_err = tag_err.or(dup_err);

    if effective_tag_err.is_some() || pubkey_err.is_some() {
        let ssh_data = build_ssh_data(
            &id,
            existing_keys,
            form,
            effective_tag_err,
            pubkey_err,
            None,
        );
        return render_detail(is_htmx, user, person, "ssh", TabContent::Ssh(ssh_data));
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let tag = form.tag.trim().to_string();
    let pubkey = form.pubkey.trim().to_string();

    match client
        .idm_person_account_post_ssh_pubkey(&id, &tag, &pubkey)
        .await
    {
        Ok(()) => {
            let fresh_entry = fetch_person(&state, &user, &id).await?;
            let fresh_person = compute_header(&fresh_entry);
            let fresh_keys = parse_keys_from_entry(&fresh_entry);
            let ssh_data =
                build_ssh_data(&id, fresh_keys, AddSshForm::default(), None, None, None);
            let toast = crate::views::toast::Toast::success("SSH key added")
                .with_desc(format!("Label: {tag}"));
            let mut resp =
                render_detail(is_htmx, user, fresh_person, "ssh", TabContent::Ssh(ssh_data))?;
            resp.headers_mut().insert("HX-Trigger", toast.hx_trigger());
            Ok(resp)
        }
        Err(e) => {
            tracing::warn!(person = %id, error = ?e, "ssh key add failed");
            let msg = friendly_client_error("add SSH key", &e);
            let ssh_data = build_ssh_data(&id, existing_keys, form, None, None, Some(msg));
            render_detail(is_htmx, user, person, "ssh", TabContent::Ssh(ssh_data))
        }
    }
}

// ── POST /people/{id}/ssh/{tag}/delete ────────────────────────────────────────

pub async fn delete(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path((id, tag)): Path<(String, String)>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let form_error = match client.idm_person_account_delete_ssh_pubkey(&id, &tag).await {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!(person = %id, tag = %tag, error = ?e, "ssh key delete failed");
            Some(friendly_client_error("delete SSH key", &e))
        }
    };

    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let keys = parse_keys_from_entry(&entry);
    let ssh_data = build_ssh_data(&id, keys, AddSshForm::default(), None, None, form_error);
    render_detail(is_htmx, user, person, "ssh", TabContent::Ssh(ssh_data))
}

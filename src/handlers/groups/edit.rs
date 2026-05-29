use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::{emails_to_rows, EmailRow};
use crate::handlers::people::create::FormField;
use crate::kanidm::entry::{attr_all, attr_first};
use crate::views::BaseFields;
use crate::AppState;

use super::common::{fetch_group, friendly_error, validate_group_name};

// ── Form data ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct EditForm {
    pub name: String,
    pub description: String,
    pub entry_managed_by: String,
    #[serde(default, rename = "email")]
    pub emails: Vec<String>,
}

// ── View ──────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "groups/edit.html")]
pub struct EditView {
    pub base: BaseFields,
    pub group_id: String,
    pub group_name: String,
    pub name_field: FormField,
    pub description_field: FormField,
    pub entry_managed_by_field: FormField,
    pub emails: Vec<EmailRow>,
    pub form_error: Option<String>,
}

impl IntoResponse for EditView {
    fn into_response(self) -> Response {
        match askama::Template::render(&self) {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::Template(e).into_response(),
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /groups/{id}/edit
pub async fn edit_form(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_group(&state, &user, &id).await?;

    let name = attr_first(&entry, "name").unwrap_or_default();
    let description = attr_first(&entry, "description").unwrap_or_default();
    let entry_managed_by = attr_first(&entry, "entry_managed_by").unwrap_or_default();
    let mails = attr_all(&entry, "mail");
    let emails = emails_to_rows(&mails);

    let form = EditForm {
        name: name.clone(),
        description,
        entry_managed_by,
        emails: mails,
    };

    Ok(build_view(&user, &id, &name, form, None, None, emails).into_response())
}

/// POST /groups/{id}
pub async fn submit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<EditForm>,
) -> AppResult<Response> {
    let trimmed_name = form.name.trim().to_string();
    let trimmed_desc = form.description.trim().to_string();
    let trimmed_mgr = form.entry_managed_by.trim().to_string();

    let mails: Vec<String> = form
        .emails
        .iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();

    let name_err = validate_group_name(&trimmed_name).err();
    let mut emails_err: Option<&'static str> = None;
    for m in &mails {
        if !m.contains('@') {
            emails_err = Some("All email addresses must contain '@'.");
            break;
        }
    }

    if name_err.is_some() || emails_err.is_some() {
        let emails_view = emails_to_rows(&form.emails);
        let name_for_label = form.name.clone();
        return Ok(build_view(&user, &id, &name_for_label, form, name_err, emails_err.map(|e| e.to_string()), emails_view).into_response());
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let mut field_errors: Vec<String> = Vec::new();

    let desc_result = if trimmed_desc.is_empty() {
        client.idm_group_purge_description(&id).await
    } else {
        client.idm_group_set_description(&id, &trimmed_desc).await
    };
    if let Err(e) = desc_result {
        tracing::warn!(error = ?e, group = %id, "failed to update group description");
        field_errors.push(friendly_error("update description", &e));
    }

    if !trimmed_mgr.is_empty()
        && let Err(e) = client.idm_group_set_entry_managed_by(&id, &trimmed_mgr).await {
            tracing::warn!(error = ?e, group = %id, "failed to update entry_managed_by");
            field_errors.push(friendly_error("update entry managed by", &e));
        }

    let mail_result = if mails.is_empty() {
        client.idm_group_purge_mail(&id).await
    } else {
        let mail_refs: Vec<&str> = mails.iter().map(|s| s.as_str()).collect();
        client.idm_group_set_mail(&id, &mail_refs).await
    };
    if let Err(e) = mail_result {
        tracing::warn!(error = ?e, group = %id, "failed to update group mail");
        field_errors.push(friendly_error("update mail", &e));
    }

    if !field_errors.is_empty() {
        let combined = field_errors.join("; ");
        let emails_view = emails_to_rows(&form.emails);
        let name_for_label = form.name.clone();
        return Ok(build_view(&user, &id, &name_for_label, form, None, Some(combined), emails_view).into_response());
    }

    if trimmed_name != id
        && let Err(e) = client.group_rename(&id, &trimmed_name).await {
            let msg = friendly_error("rename group", &e);
            tracing::warn!(error = ?e, group = %id, "kanidm rejected group rename");
            let emails_view = emails_to_rows(&form.emails);
            let name_for_label = form.name.clone();
            return Ok(build_view(&user, &id, &name_for_label, form, None, Some(msg), emails_view).into_response());
        }

    Ok(Redirect::to(&format!("/admin/groups/{trimmed_name}/overview")).into_response())
}

// ── View builder ──────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_view(
    user: &AdminUser,
    group_id: &str,
    group_name: &str,
    form: EditForm,
    name_err: Option<&'static str>,
    form_error: Option<String>,
    emails: Vec<EmailRow>,
) -> EditView {
    EditView {
        base: BaseFields::new(user, "groups"),
        group_id: group_id.to_string(),
        group_name: group_name.to_string(),
        name_field: FormField {
            id: "name",
            name: "name",
            label: "Group name",
            input_type: "text",
            value: form.name,
            placeholder: "developers",
            required: true,
            autofocus: true,
            suffix: None,
            helper: Some("Renaming a group also updates its SPN and can break references."),
            error: name_err.map(str::to_owned),
            multiline: false,
            rows: 0,
        },
        description_field: FormField {
            id: "description",
            name: "description",
            label: "Description",
            input_type: "text",
            value: form.description,
            placeholder: "Short description of this group's purpose",
            required: false,
            autofocus: false,
            suffix: None,
            helper: None,
            error: None,
            multiline: true,
            rows: 3,
        },
        entry_managed_by_field: FormField {
            id: "entry_managed_by",
            name: "entry_managed_by",
            label: "Entry managed by",
            input_type: "text",
            value: form.entry_managed_by,
            placeholder: "idm_admins",
            required: false,
            autofocus: false,
            suffix: None,
            helper: Some("The group that can manage this group's attributes and membership."),
            error: None,
            multiline: false,
            rows: 0,
        },
        emails,
        form_error,
    }
}

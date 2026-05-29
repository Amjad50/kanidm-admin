use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::{EmailRow, emails_to_rows};
use crate::handlers::people::common::validate_email_list_optional;
use crate::handlers::people::create::FormField;
use crate::views::BaseFields;

use super::common::{friendly_error, validate_description_optional, validate_group_name};

// ── Form data ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct CreateForm {
    pub name: String,
    pub entry_managed_by: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "mail")]
    pub mails: Vec<String>,
}

// ── View ──────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "groups/create.html")]
pub struct CreateView {
    pub base: BaseFields,
    pub name_field: FormField,
    pub entry_managed_by_field: FormField,
    pub description_field: FormField,
    pub emails: Vec<EmailRow>,
    pub form_error: Option<String>,
}

impl IntoResponse for CreateView {
    fn into_response(self) -> Response {
        match askama::Template::render(&self) {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::Template(e).into_response(),
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

pub async fn create_form(user: AdminUser) -> AppResult<Response> {
    Ok(build_view(&user, CreateForm::default(), None, None, None, vec![]).into_response())
}

pub async fn submit(
    State(state): State<AppState>,
    user: AdminUser,
    Form(form): Form<CreateForm>,
) -> AppResult<Response> {
    let name_err = validate_group_name(&form.name).err();
    let desc_err = validate_description_optional(&form.description).err();
    let mails_err = validate_email_list_optional(&form.mails).err();

    if name_err.is_some() || desc_err.is_some() || mails_err.is_some() {
        let emails_for_view = emails_to_rows(&form.mails);
        return Ok(
            build_view(&user, form, name_err, desc_err, mails_err, emails_for_view).into_response(),
        );
    }

    let trimmed_name = form.name.trim().to_string();
    let managed_by = form.entry_managed_by.trim().to_string();
    let managed_by_opt: Option<&str> = if managed_by.is_empty() {
        None
    } else {
        Some(&managed_by)
    };
    let trimmed_desc = form.description.trim().to_string();
    let mails: Vec<String> = form
        .mails
        .iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();

    let has_extras = !trimmed_desc.is_empty() || !mails.is_empty();

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    if let Err(e) = client.idm_group_create(&trimmed_name, managed_by_opt).await {
        let msg = friendly_error("create group", &e);
        tracing::warn!(error = ?e, "kanidm rejected group create");
        let emails_for_view = emails_to_rows(&form.mails);
        let mut view = build_view(&user, form, None, None, None, emails_for_view);
        view.form_error = Some(msg);
        return Ok(view.into_response());
    }

    tracing::info!(group = %trimmed_name, "group created");

    if has_extras {
        let mut extras_err: Option<String> = None;

        if !trimmed_desc.is_empty()
            && let Err(e) = client
                .idm_group_set_description(&trimmed_name, &trimmed_desc)
                .await
        {
            tracing::warn!(error = ?e, group = %trimmed_name, "setting description failed");
            extras_err = Some(friendly_error("set group description", &e));
        }

        if extras_err.is_none() && !mails.is_empty() {
            let mail_refs: Vec<&str> = mails.iter().map(String::as_str).collect();
            if let Err(e) = client.idm_group_set_mail(&trimmed_name, &mail_refs).await {
                tracing::warn!(error = ?e, group = %trimmed_name, "setting mail failed");
                extras_err = Some(friendly_error("set group mail", &e));
            }
        }

        if let Some(extras_msg) = extras_err {
            match client.idm_group_delete(&trimmed_name).await {
                Ok(_) => {
                    let emails_for_view = emails_to_rows(&form.mails);
                    let mut view = build_view(&user, form, None, None, None, emails_for_view);
                    view.form_error = Some(extras_msg);
                    return Ok(view.into_response());
                }
                Err(rollback_err) => {
                    tracing::error!(
                        error = ?rollback_err,
                        group = %trimmed_name,
                        "ROLLBACK FAILED after partial group create"
                    );
                    let msg = format!(
                        "{extras_msg} The group \"{trimmed_name}\" was partially created and could not be cleaned up automatically — fix it via Edit or delete it manually."
                    );
                    let emails_for_view = emails_to_rows(&form.mails);
                    let mut view = build_view(&user, form, None, None, None, emails_for_view);
                    view.form_error = Some(msg);
                    return Ok(view.into_response());
                }
            }
        }
    }

    Ok(Redirect::to(&format!("/admin/groups/{trimmed_name}/overview")).into_response())
}

// ── View builder ──────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_view(
    user: &AdminUser,
    form: CreateForm,
    name_err: Option<&'static str>,
    desc_err: Option<&'static str>,
    mails_err: Option<&'static str>,
    emails: Vec<EmailRow>,
) -> CreateView {
    let resolved_form_error = mails_err.map(str::to_owned);
    CreateView {
        base: BaseFields::new(user, "groups"),
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
            helper: Some(
                "Lowercase letters, numbers, dot, underscore, hyphen. \
                 May also start with underscore for service groups.",
            ),
            error: name_err.map(str::to_owned),
            multiline: false,
            rows: 0,
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
            helper: Some(
                "Optional. The group that can manage this group's attributes and membership. \
                 Defaults to idm_admins.",
            ),
            error: None,
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
            error: desc_err.map(str::to_owned),
            multiline: true,
            rows: 3,
        },
        emails,
        form_error: resolved_form_error,
    }
}

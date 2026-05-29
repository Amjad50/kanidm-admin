use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::people::create::FormField;
use crate::views::BaseFields;
use crate::AppState;

use super::common::{friendly_error, validate_group_name};

// ── Form data ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct CreateForm {
    pub name: String,
    pub entry_managed_by: String,
}

// ── View ──────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "groups/create.html")]
pub struct CreateView {
    pub base: BaseFields,
    pub name_field: FormField,
    pub entry_managed_by_field: FormField,
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
    Ok(build_view(&user, CreateForm::default(), None, None).into_response())
}

pub async fn submit(
    State(state): State<AppState>,
    user: AdminUser,
    Form(form): Form<CreateForm>,
) -> AppResult<Response> {
    let name_err = validate_group_name(&form.name).err();

    if name_err.is_some() {
        return Ok(build_view(&user, form, name_err, None).into_response());
    }

    let trimmed_name = form.name.trim().to_string();
    let managed_by = form.entry_managed_by.trim();
    let managed_by_opt = if managed_by.is_empty() { None } else { Some(managed_by) };

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client.idm_group_create(&trimmed_name, managed_by_opt).await {
        Ok(_) => Ok(Redirect::to(&format!("/groups/{trimmed_name}/overview")).into_response()),
        Err(e) => {
            let msg = friendly_error("create group", &e);
            tracing::warn!(error = ?e, "kanidm rejected group create");
            Ok(build_view(&user, form, None, Some(msg)).into_response())
        }
    }
}

fn build_view(
    user: &AdminUser,
    form: CreateForm,
    name_err: Option<&'static str>,
    form_error: Option<String>,
) -> CreateView {
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
        form_error,
    }
}

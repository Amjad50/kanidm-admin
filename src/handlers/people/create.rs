use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::views::BaseFields;
use crate::AppState;

use super::common::{fetch_domain_name, friendly_client_error, validate_displayname, validate_name};

// ── Form data ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct CreateForm {
    pub name: String,
    pub displayname: String,
}

// ── Form field partial ────────────────────────────────────────────────────────

/// Reusable label + input + helper + inline-error component rendered via
/// `{{ field|safe }}` in templates. Using a nested `Template` rather than
/// `{% include %}` lets each field carry its own data without polluting the
/// parent struct's namespace — which matters once the edit form adds more fields.
#[derive(Template)]
#[template(path = "people/_form_field.html")]
pub struct FormField {
    pub id: &'static str,
    pub name: &'static str,
    pub label: &'static str,
    pub input_type: &'static str,
    pub value: String,
    pub placeholder: &'static str,
    pub required: bool,
    pub autofocus: bool,
    /// Suffix badge rendered to the right of the input (e.g. "@domain.example").
    pub suffix: Option<String>,
    pub helper: Option<&'static str>,
    pub error: Option<String>,
    /// Renders a `<textarea>` instead of `<input>`. `input_type` and `suffix` are ignored.
    pub multiline: bool,
    /// Number of visible rows for the textarea. Only used when `multiline` is true.
    pub rows: u32,
}

// ── View ─────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "people/create.html")]
pub struct CreateView {
    pub base: BaseFields,
    pub name_field: FormField,
    pub displayname_field: FormField,
    /// Form-level error, e.g. kanidm rejected the request.
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

pub async fn create_form(
    State(state): State<AppState>,
    user: AdminUser,
) -> AppResult<Response> {
    let domain_name = fetch_domain_name(&state, &user).await;
    Ok(build_view(&user, CreateForm::default(), domain_name, None, None, None).into_response())
}

pub async fn submit(
    State(state): State<AppState>,
    user: AdminUser,
    Form(form): Form<CreateForm>,
) -> AppResult<Response> {
    let name_err = validate_name(&form.name).err();
    let dn_err = validate_displayname(&form.displayname).err();

    if name_err.is_some() || dn_err.is_some() {
        let domain_name = fetch_domain_name(&state, &user).await;
        return Ok(
            build_view(&user, form, domain_name, name_err, dn_err, None).into_response(),
        );
    }

    // Trim at the call site; form retains original values so re-render echoes
    // back exactly what the user typed.
    let trimmed_name = form.name.trim().to_string();
    let trimmed_displayname = form.displayname.trim().to_string();

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client
        .idm_person_account_create(&trimmed_name, &trimmed_displayname)
        .await
    {
        Ok(_) => Ok(Redirect::to(&format!("/people/{trimmed_name}/overview")).into_response()),
        Err(e) => {
            let msg = friendly_client_error("create person", &e);
            tracing::warn!(error = ?e, "kanidm rejected person create");
            let domain_name = fetch_domain_name(&state, &user).await;
            Ok(build_view(&user, form, domain_name, None, None, Some(msg)).into_response())
        }
    }
}

fn build_view(
    user: &AdminUser,
    form: CreateForm,
    domain_name: Option<String>,
    name_err: Option<&'static str>,
    dn_err: Option<&'static str>,
    form_error: Option<String>,
) -> CreateView {
    let suffix = domain_name.map(|d| format!("@{d}"));
    CreateView {
        base: BaseFields::new(user, "people"),
        name_field: FormField {
            id: "name",
            name: "name",
            label: "Username",
            input_type: "text",
            value: form.name,
            placeholder: "jane.doe",
            required: true,
            autofocus: true,
            suffix,
            helper: Some(
                "Used as the login name and in the SPN. Lowercase letters, numbers, dot, \
                 underscore, hyphen. Cannot be changed after creation without consequences.",
            ),
            error: name_err.map(str::to_owned),
            multiline: false,
            rows: 0,
        },
        displayname_field: FormField {
            id: "displayname",
            name: "displayname",
            label: "Display name",
            input_type: "text",
            value: form.displayname,
            placeholder: "Jane Doe",
            required: true,
            autofocus: false,
            suffix: None,
            helper: Some("Shown in lists and on the person's profile."),
            error: dn_err.map(str::to_owned),
            multiline: false,
            rows: 0,
        },
        form_error,
    }
}



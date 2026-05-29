use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::{emails_to_rows, EmailRow};
use crate::views::BaseFields;
use crate::AppState;

use super::common::{
    fetch_domain_name, friendly_client_error, validate_displayname, validate_email_list_optional,
    validate_legalname_optional, validate_name,
};

// ── Form data ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct CreateForm {
    pub name: String,
    pub displayname: String,
    #[serde(default)]
    pub legalname: String,
    #[serde(default, rename = "email")]
    pub emails: Vec<String>,
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
    pub legalname_field: FormField,
    pub emails: Vec<EmailRow>,
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
    Ok(build_view(&user, CreateForm::default(), domain_name, None, None, None, None, None, vec![]).into_response())
}

pub async fn submit(
    State(state): State<AppState>,
    user: AdminUser,
    Form(form): Form<CreateForm>,
) -> AppResult<Response> {
    let name_err = validate_name(&form.name).err();
    let dn_err = validate_displayname(&form.displayname).err();
    let legal_err = validate_legalname_optional(&form.legalname).err();
    let emails_err = validate_email_list_optional(&form.emails).err();

    if name_err.is_some() || dn_err.is_some() || legal_err.is_some() || emails_err.is_some() {
        let domain_name = fetch_domain_name(&state, &user).await;
        let emails_for_view = emails_to_rows(&form.emails);
        return Ok(build_view(
            &user,
            form,
            domain_name,
            name_err,
            dn_err,
            legal_err,
            emails_err,
            None,
            emails_for_view,
        )
        .into_response());
    }

    let trimmed_name = form.name.trim().to_string();
    let trimmed_dn = form.displayname.trim().to_string();
    let trimmed_legal = form.legalname.trim().to_string();
    let mails: Vec<String> = form
        .emails
        .iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();

    let has_extras = !trimmed_legal.is_empty() || !mails.is_empty();

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    if let Err(e) = client.idm_person_account_create(&trimmed_name, &trimmed_dn).await {
        let msg = friendly_client_error("create person", &e);
        tracing::warn!(error = ?e, "kanidm rejected person create");
        let domain_name = fetch_domain_name(&state, &user).await;
        let emails_for_view = emails_to_rows(&form.emails);
        return Ok(build_view(&user, form, domain_name, None, None, None, None, Some(msg), emails_for_view).into_response());
    }

    tracing::info!(person = %trimmed_name, "person account created");

    if has_extras {
        let legal_opt = if trimmed_legal.is_empty() { None } else { Some(trimmed_legal.as_str()) };
        let mail_opt: Option<&[String]> = if mails.is_empty() { None } else { Some(&mails) };

        if let Err(e) = client
            .idm_person_account_update(&trimmed_name, None, None, legal_opt, mail_opt)
            .await
        {
            tracing::warn!(error = ?e, person = %trimmed_name, "setting extras failed; rolling back person create");
            let extras_msg = friendly_client_error("set additional fields", &e);

            match client.idm_person_account_delete(&trimmed_name).await {
                Ok(_) => {
                    let domain_name = fetch_domain_name(&state, &user).await;
                    let emails_for_view = emails_to_rows(&form.emails);
                    return Ok(build_view(&user, form, domain_name, None, None, None, None, Some(extras_msg), emails_for_view).into_response());
                }
                Err(rollback_err) => {
                    tracing::error!(
                        error = ?rollback_err,
                        person = %trimmed_name,
                        "ROLLBACK FAILED after partial person create"
                    );
                    let msg = format!(
                        "{extras_msg} The account \"{trimmed_name}\" was partially created and could not be cleaned up automatically — fix it via Edit or delete it manually."
                    );
                    let domain_name = fetch_domain_name(&state, &user).await;
                    let emails_for_view = emails_to_rows(&form.emails);
                    return Ok(build_view(&user, form, domain_name, None, None, None, None, Some(msg), emails_for_view).into_response());
                }
            }
        }
    }

    Ok(Redirect::to(&format!("/admin/people/{trimmed_name}/overview")).into_response())
}

// ── View builder ─────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_view(
    user: &AdminUser,
    form: CreateForm,
    domain_name: Option<String>,
    name_err: Option<&'static str>,
    dn_err: Option<&'static str>,
    legal_err: Option<&'static str>,
    emails_err: Option<&'static str>,
    form_error: Option<String>,
    emails: Vec<EmailRow>,
) -> CreateView {
    let suffix = domain_name.map(|d| format!("@{d}"));
    // Merge field-level email error into form_error if present and no form_error yet.
    let resolved_form_error = form_error.or_else(|| emails_err.map(str::to_owned));
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
        legalname_field: FormField {
            id: "legalname",
            name: "legalname",
            label: "Legal name",
            input_type: "text",
            value: form.legalname,
            placeholder: "",
            required: false,
            autofocus: false,
            suffix: None,
            helper: Some("Optional. Used in reports and audit logs."),
            error: legal_err.map(str::to_owned),
            multiline: false,
            rows: 0,
        },
        emails,
        form_error: resolved_form_error,
    }
}

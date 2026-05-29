use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::common::{EmailRow, emails_to_rows};
use crate::kanidm::entry::{attr_all, attr_first};
use crate::views::BaseFields;

use super::common::{
    fetch_domain_name, friendly_client_error, validate_displayname, validate_name,
};
use super::create::FormField;

// ── Form data ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct EditForm {
    pub name: String,
    pub displayname: String,
    pub legalname: String,
    #[serde(default, rename = "email")]
    pub emails: Vec<String>,
}

#[derive(Template)]
#[template(path = "people/edit.html")]
pub struct EditView {
    pub base: BaseFields,
    pub person_id: String,
    pub person_displayname: String,
    pub name_field: FormField,
    pub displayname_field: FormField,
    pub legalname_field: FormField,
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

/// GET /people/{id}/edit — render edit form prefilled with current values.
pub async fn edit_form(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let entry = client
        .idm_person_account_get(&id)
        .await
        .map_err(|e| AppError::Kanidm(format!("person get failed: {e:?}")))?
        .ok_or(AppError::NotFound)?;

    let name = attr_first(&entry, "name").unwrap_or_default();
    let displayname = attr_first(&entry, "displayname")
        .or_else(|| attr_first(&entry, "name"))
        .unwrap_or_default();
    let legalname = attr_first(&entry, "legalname").unwrap_or_default();
    let mails = attr_all(&entry, "mail");

    let domain_suffix = fetch_domain_name(&state, &user).await;

    let form = EditForm {
        name: name.clone(),
        displayname: displayname.clone(),
        legalname,
        emails: mails.clone(),
    };

    let emails = emails_to_rows(&mails);

    Ok(build_view(
        &user,
        &id,
        &displayname,
        form,
        domain_suffix,
        None,
        None,
        None,
        emails,
    )
    .into_response())
}

/// POST /people/{id} — submit edits, redirect to /people/{new_name}/overview on success.
pub async fn submit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<EditForm>,
) -> AppResult<Response> {
    let trimmed_name = form.name.trim().to_string();
    let trimmed_dn = form.displayname.trim().to_string();
    let trimmed_ln = form.legalname.trim().to_string();

    let mails: Vec<String> = form
        .emails
        .iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();

    let name_err = validate_name(&trimmed_name).err();
    let dn_err = validate_displayname(&trimmed_dn).err();
    let mut emails_err: Option<&'static str> = None;
    for m in &mails {
        if !m.contains('@') {
            emails_err = Some("All emails must contain '@'.");
            break;
        }
    }

    if name_err.is_some() || dn_err.is_some() || emails_err.is_some() {
        let domain_suffix = fetch_domain_name(&state, &user).await;
        let emails_for_view = emails_to_rows(&form.emails);
        // Why: form.displayname is the raw user input — correct for the page
        // title whether the error is on displayname (trimmed is empty) or on
        // another field (trimmed equals form.displayname anyway).
        let person_displayname = form.displayname.clone();
        return Ok(build_view(
            &user,
            &id,
            &person_displayname,
            form,
            domain_suffix,
            name_err,
            dn_err,
            emails_err.map(|e| e.to_string()),
            emails_for_view,
        )
        .into_response());
    }

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    match client
        .idm_person_account_update(
            &id,
            Some(&trimmed_name),
            Some(&trimmed_dn),
            Some(&trimmed_ln),
            Some(&mails),
        )
        .await
    {
        Ok(()) => {
            Ok(Redirect::to(&format!("/admin/people/{trimmed_name}/overview")).into_response())
        }
        Err(e) => {
            let msg = friendly_client_error("update person", &e);
            tracing::warn!(error = ?e, person = %id, "kanidm rejected person update");
            let domain_suffix = fetch_domain_name(&state, &user).await;
            let emails_for_view = emails_to_rows(&form.emails);
            Ok(build_view(
                &user,
                &id,
                &trimmed_dn,
                form,
                domain_suffix,
                None,
                None,
                Some(msg),
                emails_for_view,
            )
            .into_response())
        }
    }
}

// ── View builder ─────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_view(
    user: &AdminUser,
    person_id: &str,
    person_displayname: &str,
    form: EditForm,
    domain_suffix: Option<String>,
    name_err: Option<&'static str>,
    dn_err: Option<&'static str>,
    form_error: Option<String>,
    emails: Vec<EmailRow>,
) -> EditView {
    let suffix = domain_suffix.map(|d| format!("@{d}"));
    EditView {
        base: BaseFields::new(user, "people"),
        person_id: person_id.to_string(),
        person_displayname: person_displayname.to_string(),
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
            helper: None,
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
            helper: Some("Used in reports and audit logs. Leave empty to clear."),
            error: None,
            multiline: false,
            rows: 0,
        },
        emails,
        form_error,
    }
}

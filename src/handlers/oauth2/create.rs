use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::handlers::people::create::FormField;
use crate::views::BaseFields;

use super::common::{
    OAuth2CreateKind, validate_landing_url, validate_oauth2_displayname, validate_oauth2_name,
};
use crate::handlers::common::friendly_client_error;

// ── Query / form structs ──────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct DetailsQuery {
    #[serde(rename = "type")]
    pub kind: Option<OAuth2CreateKind>,
}

#[derive(serde::Deserialize)]
pub struct SubmitForm {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub displayname: String,
    #[serde(default)]
    pub landing: String,
    #[serde(rename = "type", default = "default_kind")]
    pub kind: OAuth2CreateKind,
}

fn default_kind() -> OAuth2CreateKind {
    OAuth2CreateKind::Basic
}

impl Default for SubmitForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            displayname: String::new(),
            landing: String::new(),
            kind: OAuth2CreateKind::Basic,
        }
    }
}

// ── View structs ──────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "oauth2/new.html")]
pub struct PickTypeView {
    pub base: BaseFields,
    pub selected: Option<OAuth2CreateKind>,
}

#[derive(Template, WebTemplate)]
#[template(path = "oauth2/new_details.html")]
pub struct DetailsView {
    pub base: BaseFields,
    pub kind: OAuth2CreateKind,
    pub name_field: FormField,
    pub displayname_field: FormField,
    pub landing_field: FormField,
    pub form_error: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /oauth2/new — step 1: choose client type.
pub async fn pick_type(user: AdminUser) -> AppResult<Response> {
    Ok(PickTypeView {
        base: BaseFields::new(&user, "oauth2"),
        selected: None,
    }
    .into_response())
}

/// GET /oauth2/new/details?type=basic|public — step 2: fill in details.
pub async fn details_form(user: AdminUser, Query(q): Query<DetailsQuery>) -> AppResult<Response> {
    let Some(kind) = q.kind else {
        return Ok(Redirect::to("/admin/oauth2/new").into_response());
    };
    Ok(
        build_details_view(&user, kind, SubmitForm::default(), None, None, None, None)
            .into_response(),
    )
}

/// POST /oauth2 — validate + create.
pub async fn submit(
    State(state): State<AppState>,
    user: AdminUser,
    Form(form): Form<SubmitForm>,
) -> AppResult<Response> {
    let name_err = validate_oauth2_name(&form.name).err();
    let dn_err = validate_oauth2_displayname(&form.displayname).err();
    let landing_err = validate_landing_url(&form.landing).err();

    if name_err.is_some() || dn_err.is_some() || landing_err.is_some() {
        return Ok(
            build_details_view(&user, form.kind, form, name_err, dn_err, landing_err, None)
                .into_response(),
        );
    }

    let trimmed_name = form.name.trim().to_string();
    let trimmed_dn = form.displayname.trim().to_string();
    let trimmed_landing = form.landing.trim().to_string();
    let kind = form.kind;

    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let result = match kind {
        OAuth2CreateKind::Basic => {
            client
                .idm_oauth2_rs_basic_create(&trimmed_name, &trimmed_dn, &trimmed_landing)
                .await
        }
        OAuth2CreateKind::Public => {
            client
                .idm_oauth2_rs_public_create(&trimmed_name, &trimmed_dn, &trimmed_landing)
                .await
        }
    };

    if let Err(e) = result {
        let msg = friendly_client_error("create oauth2 client", &e);
        tracing::warn!(name = %trimmed_name, error = ?e, "oauth2 create failed");
        return Ok(build_details_view(
            &user,
            kind,
            SubmitForm {
                name: trimmed_name,
                displayname: trimmed_dn,
                landing: trimmed_landing,
                kind,
            },
            None,
            None,
            None,
            Some(msg),
        )
        .into_response());
    }

    tracing::info!(name = %trimmed_name, kind = ?kind, "oauth2 client created");
    Ok(Redirect::to(&format!("/admin/oauth2/{trimmed_name}/general")).into_response())
}

// ── View builder ──────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_details_view(
    user: &AdminUser,
    kind: OAuth2CreateKind,
    form: SubmitForm,
    name_err: Option<&'static str>,
    dn_err: Option<&'static str>,
    landing_err: Option<&'static str>,
    form_error: Option<String>,
) -> DetailsView {
    DetailsView {
        base: BaseFields::new(user, "oauth2"),
        kind,
        name_field: FormField {
            id: "name",
            name: "name",
            label: "Client name",
            input_type: "text",
            value: form.name,
            placeholder: "grafana",
            required: true,
            autofocus: true,
            suffix: None,
            helper: Some(
                "Used in URLs and tokens. Lowercase letters, digits, '.', '_', '-'. \
                 Must start with a lowercase letter. Max 63 characters.",
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
            placeholder: "Grafana",
            required: true,
            autofocus: false,
            suffix: None,
            helper: Some("Shown on consent screens and the app listing."),
            error: dn_err.map(str::to_owned),
            multiline: false,
            rows: 0,
        },
        landing_field: FormField {
            id: "landing",
            name: "landing",
            label: "Landing URL",
            input_type: "url",
            value: form.landing,
            placeholder: "https://grafana.example.com",
            required: true,
            autofocus: false,
            suffix: None,
            helper: Some(
                "The application's primary URL. Becomes the landing URL and default redirect origin.",
            ),
            error: landing_err.map(str::to_owned),
            multiline: false,
            rows: 0,
        },
        form_error,
    }
}

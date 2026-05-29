use axum::Form;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Response};
use axum_htmx::HxRequest;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::AppState;
use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::views::{format_absolute, format_relative_future, format_relative_past};

use super::common::{
    PersonStatus, compute_status_at, friendly_client_error, parse_kanidm_datetime,
};
use super::detail::{TabContent, compute_header, fetch_person, render_detail};

// ── View model ────────────────────────────────────────────────────────────────

pub enum ValidityMode {
    Clear,
    Datetime,
}

pub struct ValidityField {
    pub form_action: String,
    pub current_mode: ValidityMode,
    pub date_value: String,
    pub time_value: String,
    pub clear_label: &'static str,
    pub clear_description: &'static str,
}

pub struct ValidityData {
    pub status: PersonStatus,
    pub status_message: String,
    pub status_classes: &'static str,
    pub valid_from: ValidityField,
    pub expire: ValidityField,
    pub error: Option<String>,
}

// ── Form deserializer ─────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct ValidityForm {
    pub mode: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub time: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn split_datetime(dt: OffsetDateTime) -> (String, String) {
    let dt = dt.to_offset(time::UtcOffset::UTC);
    let date = format!("{:04}-{:02}-{:02}", dt.year(), dt.month() as u8, dt.day());
    let time = format!("{:02}:{:02}", dt.hour(), dt.minute());
    (date, time)
}

fn build_validity_field(
    raw: Option<String>,
    field_name: &'static str,
    person_id: &str,
    clear_label: &'static str,
    clear_description: &'static str,
) -> ValidityField {
    let form_action = format!("/admin/people/{person_id}/validity/{field_name}");
    match raw.as_deref().and_then(parse_kanidm_datetime) {
        Some(dt) => {
            let (date_value, time_value) = split_datetime(dt);
            ValidityField {
                form_action,
                current_mode: ValidityMode::Datetime,
                date_value,
                time_value,
                clear_label,
                clear_description,
            }
        }
        None => ValidityField {
            form_action,
            current_mode: ValidityMode::Clear,
            date_value: String::new(),
            time_value: String::new(),
            clear_label,
            clear_description,
        },
    }
}

fn build_status_message(
    entry: &kanidm_proto::v1::Entry,
    now: OffsetDateTime,
) -> (PersonStatus, String, &'static str) {
    use crate::kanidm::entry::attr_first;

    let status = compute_status_at(entry, now);

    let valid_from_dt = attr_first(entry, "account_valid_from")
        .as_deref()
        .and_then(parse_kanidm_datetime);
    let expire_dt = attr_first(entry, "account_expire")
        .as_deref()
        .and_then(parse_kanidm_datetime);

    let start_label = match valid_from_dt {
        Some(dt) => format!("{} ({})", format_absolute(dt), format_relative_past(dt)),
        None => "any time".to_string(),
    };
    let end_label = match expire_dt {
        Some(dt) => format!("{} ({})", format_absolute(dt), format_relative_future(dt)),
        None => "forever".to_string(),
    };

    let (message, classes) = match status {
        PersonStatus::Active => (
            format!("Account is valid from {start_label} until {end_label}."),
            "bg-success-soft border-success text-success",
        ),
        PersonStatus::NotYetActive => (
            format!(
                "Not yet active. Becomes valid {}.",
                match valid_from_dt {
                    Some(dt) => format!("{} ({})", format_relative_future(dt), format_absolute(dt)),
                    None => "any time".to_string(),
                }
            ),
            "bg-warning-soft border-warning text-warning",
        ),
        PersonStatus::Expired => (
            format!(
                "Expired {}.",
                match expire_dt {
                    Some(dt) => format!("{} ({})", format_relative_past(dt), format_absolute(dt)),
                    None => "at an unknown time".to_string(),
                }
            ),
            "bg-danger-soft border-danger text-danger",
        ),
    };

    (status, message, classes)
}

fn build_validity_data(
    entry: &kanidm_proto::v1::Entry,
    person_id: String,
    error: Option<String>,
) -> ValidityData {
    use crate::kanidm::entry::attr_first;

    let now = OffsetDateTime::now_utc();
    let (status, status_message, status_classes) = build_status_message(entry, now);

    let valid_from = build_validity_field(
        attr_first(entry, "account_valid_from"),
        "valid_from",
        &person_id,
        "Any time",
        "No lower bound — the account is valid right now.",
    );
    let expire = build_validity_field(
        attr_first(entry, "account_expire"),
        "expire",
        &person_id,
        "Never",
        "No upper bound — the account stays active indefinitely.",
    );

    ValidityData {
        status,
        status_message,
        status_classes,
        valid_from,
        expire,
        error,
    }
}

fn render_validity_fragment(
    person: super::detail::PersonHeader,
    tab_content: TabContent,
) -> AppResult<Response> {
    use super::detail::TabContentFragment;

    let html = askama::Template::render(&TabContentFragment {
        tab_content: &tab_content,
        person: &person,
    })
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

// ── GET /people/{id}/validity ─────────────────────────────────────────────────

pub async fn tab(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    Path(id): Path<String>,
    user: AdminUser,
) -> AppResult<Response> {
    let entry = fetch_person(&state, &user, &id).await?;
    let person = compute_header(&entry);
    let data = build_validity_data(&entry, id, None);
    let tab_content = TabContent::Validity(data);
    render_detail(is_htmx, user, person, "validity", tab_content)
}

// ── POST /people/{id}/validity/valid_from ─────────────────────────────────────

pub async fn set_valid_from(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<ValidityForm>,
) -> AppResult<Response> {
    apply_validity_change(&state, &id, user, form, "valid_from", "account_valid_from").await
}

// ── POST /people/{id}/validity/expire ─────────────────────────────────────────

pub async fn set_expire(
    State(state): State<AppState>,
    Path(id): Path<String>,
    user: AdminUser,
    Form(form): Form<ValidityForm>,
) -> AppResult<Response> {
    apply_validity_change(&state, &id, user, form, "expire", "account_expire").await
}

// ── Shared mutation logic ─────────────────────────────────────────────────────

async fn apply_validity_change(
    state: &AppState,
    id: &str,
    user: AdminUser,
    form: ValidityForm,
    _field_slug: &str,
    attr_name: &str,
) -> AppResult<Response> {
    let client = state
        .kanidm
        .for_token(&user.token)
        .await
        .map_err(|e| AppError::Kanidm(e.to_string()))?;

    let mutation_error: Option<String> = match form.mode.as_str() {
        "clear" => match client.idm_person_account_purge_attr(id, attr_name).await {
            Ok(()) => None,
            Err(e) => {
                tracing::warn!(person = %id, attr = %attr_name, error = ?e, "purge attr failed");
                Some(friendly_client_error("clear attribute", &e))
            }
        },
        "datetime" => {
            if form.date.is_empty() || form.time.is_empty() {
                Some("Date and time are both required for a specific date.".to_string())
            } else {
                let combined = format!("{}T{}:00Z", form.date, form.time);
                match OffsetDateTime::parse(&combined, &Rfc3339) {
                    Ok(_) => {
                        match client
                            .idm_person_account_set_attr(id, attr_name, &[&combined])
                            .await
                        {
                            Ok(()) => None,
                            Err(e) => {
                                tracing::warn!(person = %id, attr = %attr_name, error = ?e, "set attr failed");
                                Some(friendly_client_error("set attribute", &e))
                            }
                        }
                    }
                    Err(_) => Some("Invalid date or time.".to_string()),
                }
            }
        }
        _ => Some("Invalid mode.".to_string()),
    };

    let entry = fetch_person(state, &user, id).await?;
    let person = compute_header(&entry);
    let data = build_validity_data(&entry, id.to_string(), mutation_error);

    render_validity_fragment(person, TabContent::Validity(data))
}

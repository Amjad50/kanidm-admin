use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};

use crate::error::{AppError, AppResult};
use crate::views::partials::Modal;
use crate::AppState;

#[derive(Template)]
#[template(path = "reauth_modal.html")]
struct ReauthBody {}

#[derive(Template)]
#[template(path = "reauth_modal_footer.html")]
struct ReauthFooter {
    login_url: String,
}

pub async fn reauth(State(state): State<AppState>) -> AppResult<Response> {
    let body_html = ReauthBody {}.render().map_err(AppError::Template)?;
    let footer_html = ReauthFooter {
        login_url: format!("{}/ui/login", state.config.kanidm_url.trim_end_matches('/')),
    }
    .render()
    .map_err(AppError::Template)?;

    let html = Modal {
        title: "Session expired".to_string(),
        icon_name: None,
        icon_color_class: "text-tertiary",
        body_html,
        footer_html,
        size_class: "max-w-md",
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(html).into_response())
}

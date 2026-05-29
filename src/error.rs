use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::views::{ForbiddenView, NotFoundView, ServerErrorView, UnauthenticatedView};

/// App-level error type. Converts to an HTTP response.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not authenticated")]
    Unauthenticated {
        kanidm_url: String,
        /// Set when the request came from HTMX. Triggers the reauth modal
        /// via an HX-Trigger header instead of replacing the page with 401.
        is_htmx: bool,
    },

    #[error("forbidden: not a member of the admin group")]
    Forbidden { admin_group: String },

    #[error("not found")]
    NotFound,

    #[error("kanidm client error: {0}")]
    Kanidm(String),

    #[error("template render error: {0}")]
    Template(#[from] askama::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthenticated { kanidm_url, is_htmx } => {
                if is_htmx {
                    let mut resp = StatusCode::OK.into_response();
                    resp.headers_mut().insert(
                        "HX-Trigger",
                        HeaderValue::from_static(r#"{"kanidm-reauth":null}"#),
                    );
                    return resp;
                }
                let view = UnauthenticatedView { kanidm_url };
                (StatusCode::UNAUTHORIZED, view.into_response()).into_response()
            }
            AppError::Forbidden { admin_group } => {
                let view = ForbiddenView { admin_group };
                (StatusCode::FORBIDDEN, view.into_response()).into_response()
            }
            AppError::NotFound => {
                let view = NotFoundView {};
                (StatusCode::NOT_FOUND, view.into_response()).into_response()
            }
            AppError::Kanidm(msg) => {
                tracing::error!(error = %msg, "kanidm client error");
                let view = ServerErrorView { category: "Kanidm API error" };
                (StatusCode::BAD_GATEWAY, view.into_response()).into_response()
            }
            AppError::Template(err) => {
                tracing::error!(error = %err, "template render error");
                let view = ServerErrorView { category: "Template render error" };
                (StatusCode::INTERNAL_SERVER_ERROR, view.into_response()).into_response()
            }
            AppError::Other(err) => {
                tracing::error!(error = ?err, "unhandled error");
                let view = ServerErrorView { category: "Server error" };
                (StatusCode::INTERNAL_SERVER_ERROR, view.into_response()).into_response()
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

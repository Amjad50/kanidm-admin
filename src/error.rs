use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::views::{ForbiddenView, ServerErrorView, UnauthenticatedView};

/// App-level error type. Converts to an HTTP response.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not authenticated")]
    Unauthenticated { kanidm_url: String },

    #[error("forbidden: not a member of the admin group")]
    Forbidden { admin_group: String },

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
            AppError::Unauthenticated { kanidm_url } => {
                let view = UnauthenticatedView { kanidm_url };
                (StatusCode::UNAUTHORIZED, view.into_response()).into_response()
            }
            AppError::Forbidden { admin_group } => {
                let view = ForbiddenView { admin_group };
                (StatusCode::FORBIDDEN, view.into_response()).into_response()
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

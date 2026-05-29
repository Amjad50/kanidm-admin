use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// App-level error type. Converts to an HTTP response.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not authenticated")]
    Unauthenticated,

    #[error("forbidden: not a member of the admin group")]
    Forbidden,

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
            // For now, both unauth and forbidden return generic responses.
            // Phase 2 will wire these to the kanidm login redirect + an HTML "forbidden" page.
            AppError::Unauthenticated => {
                (StatusCode::UNAUTHORIZED, "Unauthenticated. Sign in to kanidm first.").into_response()
            }
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Forbidden. Your account is not in the admin group.",
            )
                .into_response(),
            AppError::Kanidm(msg) => {
                tracing::error!(error = %msg, "kanidm client error");
                (StatusCode::BAD_GATEWAY, format!("Upstream error: {msg}")).into_response()
            }
            AppError::Template(err) => {
                tracing::error!(error = %err, "template render error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
            }
            AppError::Other(err) => {
                tracing::error!(error = ?err, "unhandled error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

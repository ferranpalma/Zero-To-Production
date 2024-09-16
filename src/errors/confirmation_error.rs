use actix_web::{http::StatusCode, ResponseError};

use crate::errors::format_error_chain;

#[derive(thiserror::Error)]
pub enum ConfirmationError {
    #[error("{0}")]
    TokenError(String),
    #[error("There is no subscriber associated with the provided token")]
    UnknownToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfirmationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ConfirmationError::TokenError(_) => StatusCode::BAD_REQUEST,
            ConfirmationError::UnknownToken => StatusCode::UNAUTHORIZED,
            ConfirmationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_error_chain(self, f)
    }
}

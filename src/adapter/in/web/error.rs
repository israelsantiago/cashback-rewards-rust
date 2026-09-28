use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

use crate::application::ApplicationError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("{0}")]
    Application(#[from] ApplicationError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            ApiError::Validation(message) => (StatusCode::BAD_REQUEST, json!({"message": message})),
            ApiError::Application(ApplicationError::Domain(
                crate::domain::exception::DomainError::MerchantAlreadyRegistered(_),
            )) => (StatusCode::CONFLICT, json!({"message": self.to_string()})),
            ApiError::Application(ApplicationError::Domain(
                crate::domain::exception::DomainError::DefaultRateNotConfigured,
            )) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({"message": self.to_string()}),
            ),
            ApiError::Application(ApplicationError::Domain(
                crate::domain::exception::DomainError::MerchantNotFound(_),
            )) => (StatusCode::NOT_FOUND, json!({"message": self.to_string()})),
            ApiError::Application(ApplicationError::Port(_)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": "internal server error"}),
            ),
        };

        (status, Json(body)).into_response()
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone)]
pub struct EndpointError {
    pub status: StatusCode,
    pub message: String,
    pub error_code: Option<String>,  // For API error codes
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

impl EndpointError {
    // Constructors for common errors
    pub fn bad_request(msg: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.to_string(),
            error_code: None,
        }
    }

    pub fn unauthorized(msg: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: msg.to_string(),
            error_code: None,
        }
    }

    pub fn forbidden(msg: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: msg.to_string(),
            error_code: None,
        }
    }

    pub fn not_found(msg: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.to_string(),
            error_code: None,
        }
    }

    pub fn conflict(msg: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: msg.to_string(),
            error_code: None,
        }
    }

    pub fn internal_server_error(msg: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.to_string(),
            error_code: None,
        }
    }

    pub fn unprocessable_entity(msg: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: msg.to_string(),
            error_code: None,
        }
    }

    // Constructor with custom error code
    pub fn with_code(mut self, code: &str) -> Self {
        self.error_code = Some(code.to_string());
        self
    }

    // Constructor with custom status
    pub fn custom(status: StatusCode, msg: &str) -> Self {
        Self {
            status,
            message: msg.to_string(),
            error_code: None,
        }
    }
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {}: {}", self.status.as_u16(), self.message)
    }
}

impl std::error::Error for EndpointError {}

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            error: self.status.canonical_reason().unwrap_or("Unknown Error").to_string(),
            message: self.message.clone(),
            code: self.error_code.clone(),
        };

        (self.status, Json(error_response)).into_response()
    }
}

// Convenient conversion from service errors
impl From<crate::services::users::UserDatabaseError> for EndpointError {
    fn from(err: crate::services::users::UserDatabaseError) -> Self {
        match err {
            crate::services::users::UserDatabaseError::NotFound(msg) => {
                        EndpointError::not_found(&msg)
                    }
            crate::services::users::UserDatabaseError::ValidationError(msg) => {
                        EndpointError::bad_request(&msg)
                    }
            crate::services::users::UserDatabaseError::DatabaseError(_) => {
                        EndpointError::internal_server_error("Database error occurred")
                    }
            _ => EndpointError::internal_server_error("An error occured during your request")
        }
    }
}
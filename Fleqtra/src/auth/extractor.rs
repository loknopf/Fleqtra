use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode, header::AUTHORIZATION}, response::IntoResponse};
use crate::auth::claims::Claims;
use crate::auth::jwt::validate_jwt;
pub struct AuthenticatedUser(pub Claims);

#[derive(Debug)]
pub struct AuthError {
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, self.message).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AuthError {
                message: "Missing Authorization header".to_string(),
            })?;

        let auth_str = authorization
            .to_str()
            .map_err(|_| AuthError {
                message: "Invalid Authorization header format".to_string(),
            })?;

        if !auth_str.starts_with("Bearer ") {
            return Err(AuthError {
                message: "Authorization header must start with 'Bearer '".to_string(),
            });
        }

        let token = &auth_str[7..]; // "Bearer ".len() = 7

        let claims = validate_jwt(token).map_err(|_| AuthError {
            message: "Invalid or expired JWT token".to_string(),
        })?;

        Ok(AuthenticatedUser(claims))
    }
}
use axum::{
    extract::{Json, State}, response::IntoResponse, routing::{post, Route}, Router
};
use serde::{Deserialize, Serialize};

use crate::{auth::{create_jwt, AuthenticatedUser}, models::user::User, services::users::UserService, utils::endpoint::EndpointError};
use crate::utils::user::{verify_password};

//Request and Response structs for de-/serialization
#[derive(Debug, Deserialize)]
struct  LoginRequest{
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse{
    jwt: String
}

#[derive(Debug, Serialize)]
struct TokenRefreshResponse{
    jwt: String
}

//Route generator
pub fn routes() -> Router<UserService>{
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh))
}

//Endpoint funcs
async fn login(State(user_service): State<UserService>,
                   Json(payload): Json<LoginRequest>) -> impl IntoResponse{
    //validate login here
    let hashed_pw = match user_service.get_user_password(&payload.username).await{
        Ok(pw) => pw,
        Err(_) => return EndpointError::internal_server_error("Error while fetching user credentials").into_response(),
    };
    let user = match user_service.get_user_by_username(&payload.username).await{
        Ok(u) => u,
        Err(_) => return EndpointError::internal_server_error("Error while fetching user").into_response(),
    };
    match verify_password(&payload.password, &hashed_pw) {
        Ok(value) => {
            if value == true {
                let token = match create_jwt(user.id, user.email, user.username){
                    Ok(value) => value,
                    Err(_) => return EndpointError::internal_server_error("Error while creating token.").into_response(),
                };
                return Json(LoginResponse { jwt: token }).into_response();
            }else {
                return EndpointError::unauthorized("Password not correct.").into_response();
            }
        },
        Err(_) => return EndpointError::internal_server_error("Error during user credentials verification process.").into_response(),
    }
}

async fn logout(State(user_service): State<UserService>,
                AuthenticatedUser(claims): AuthenticatedUser) -> impl IntoResponse{
                    format!("{} logged out successfully!", claims.username)
                }

async  fn refresh(State(user_service): State<UserService>,
                  AuthenticatedUser(claims): AuthenticatedUser) -> impl IntoResponse{
                    let refresh_jwt = match create_jwt(claims.id, claims.email, claims.username){
                        Ok(j) => j,
                        Err(e) => return EndpointError::internal_server_error("Unable to create new token").into_response(),
                    };
                    Json(TokenRefreshResponse{ jwt: refresh_jwt }).into_response()
                  }
use axum::{
    extract::{Json, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Router
};
use serde::{Deserialize, Serialize};

use crate::{models::user::User, services::users::UserService, utils::endpoint::EndpointError};
use crate::utils::user::{hash_password};

//Requests and Responses
#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct CreateUserResponse {
    user_id: Option<i32>
} 

#[derive(Debug, Deserialize)]
struct GetUserRequest{
    username: Option<String>,
    email: Option<String>,
    user_id: Option<i32>
}

#[derive(Debug, Serialize)]
struct GetUserResponse{
    email: String,
    username: String,
    containers: Vec<String>
}

//Helper
pub enum GetUserSelector {
    ByUsername(String),
    ByEmail(String),
    ById(i32),
}

impl TryFrom<GetUserRequest> for GetUserSelector {
    type Error = EndpointError;          // your own error type (see below)

    fn try_from(req: GetUserRequest) -> Result<Self, Self::Error> {
        match (req.username, req.email, req.user_id) {
            // exactly one selector?
            (Some(u), None, None) => Ok(Self::ByUsername(u)),
            (None, Some(e), None) => Ok(Self::ByEmail(e)),
            (None, None, Some(id)) => Ok(Self::ById(id)),
            _ => Err(EndpointError::bad_request(
                "provide exactly one of username, email or user_id",
            )),
        }
    }
}

//Route generator
pub fn routes() -> Router<UserService>{
    Router::new()
        .route("/users", get(get_user))
        .route("/users", post(create_user))
}

async fn create_user(State(user_service): State<UserService>,
                     Json(payload): Json<CreateUserRequest>) -> impl IntoResponse{
    let hashed_pw = match hash_password(&payload.password){
        Ok(pw) => pw,
        Err(err) => return EndpointError::internal_server_error(&format!("Could not hash password due to {}.", err)).into_response(),
    };
    let user_id = match user_service.create_user(payload.username, payload.email, hashed_pw).await {
        Ok(id) => id,
        Err(e) => return EndpointError::internal_server_error(&e.to_string()).into_response(),
    };
    let response = CreateUserResponse{ user_id: Some(user_id) };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn get_user(State(user_service): State<UserService>,
    Json(payload): Json<GetUserRequest>) -> impl IntoResponse{
    let selector = match GetUserSelector::try_from(payload) {
        Ok(sel) => sel,
        Err(e) => return EndpointError::internal_server_error(&e.to_string()).into_response(),
    };
    let user:User;
    match selector {
        GetUserSelector::ByUsername(username) => {
            user = match user_service.get_user_by_username(&username).await {
                Ok(u) => u,
                Err(e) => return EndpointError::internal_server_error(&e.to_string()).into_response(), 
            };
        },
        GetUserSelector::ByEmail(_) => todo!(),
        GetUserSelector::ById(id) => {
            user = match user_service.get_user_by_id(&id).await {
                Ok(u) => u,
                Err(e) => return EndpointError::internal_server_error("Failed to get user.").into_response(), 
            }
        },
    }
    let response = GetUserResponse { username: user.username, email: user.email, containers: user.containers };

    Json(response).into_response()
}
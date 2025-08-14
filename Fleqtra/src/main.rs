use std::{path::Path};
use std::env;
use axum::http::HeaderMap;
use axum::{
    extract::{Json, State}, 
    http::StatusCode, 
    response::{IntoResponse}, 
    routing::{get, post}, 
    Router
};

use tokio::fs;

use axum_extra::extract::Multipart;

use serde::{Deserialize, Serialize};

use dotenvy;

use deadpool_postgres::{
    Config, Manager, ManagerConfig, Pool, RecyclingMethod,
};

use tokio_postgres::{NoTls};

mod models;
mod routes;
mod services;
mod utils;
mod auth;

use crate::models::user::User;

#[derive(Deserialize, Debug)]
struct CreateUser{
    username: String,
}

#[derive(Deserialize, Debug)]
struct QueryUser{
    user_id: i32
}

const STORAGE_DIR: &str = "./uploads";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    fs::create_dir_all(STORAGE_DIR).await.unwrap();

    dotenvy::from_path(Path::new("./config/.env")).ok();

    //Database setup
    //let db_connection_string = env::var("DATABASE_CONNECTION_STRING").expect("Database connection string not in enviroment");
    let mut cfg = Config::new();
    cfg.dbname = Some(String::from("postgres"));
    cfg.user = Some(String::from("postgres"));
    cfg.password = Some(String::from("Lorenz1412"));
    cfg.host = Some(String::from("localhost"));
    cfg.port = Some(5432);
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    let pool: Pool = cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)?;

    let address_str = env::var("ADDRESS").expect("Address not set in enviroment");
    let port: u16 = env::var("PORT")
                         .expect("Port not set in enviroment")
                         .parse()
                         .expect("Port must be number");

    let app = Router::new().route("/", get(|| async { "Hello World" }))
                                                  .route("/user", get(get_user))
                                                  .route("/user", post(create_user))
                                                  .route("/upload", post(upload_file))
                                                  .with_state(pool);
    let bind_address = format!("{}:{}", address_str, port);
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn upload_file(State(pool): State<Pool>,
                     header: HeaderMap,
                     mut multipart: Multipart
                     ) -> impl IntoResponse{
    let user_id_opt = header.get("user_id").and_then(|v| v.to_str().ok());
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(filename) = field.file_name().map(|s| s.to_string()) {
            let user_id: i32 = match user_id_opt {
                Some(id) => id.parse().unwrap(),
                None => return (StatusCode::BAD_REQUEST, "No user_id value in header".to_string()).into_response(),
            };
            let data = field.bytes().await.unwrap();
            let folder_path = format!("{}/{}/", STORAGE_DIR, user_id);
            let folder_result = fs::create_dir_all(folder_path).await;
            match folder_result {
                Ok(_) => {},
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
            let path = format!("{}/{}/{}", STORAGE_DIR, user_id, sanitize_filename(&filename));
            if let Err(e) = fs::write(&path, &data).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            let client = match pool.get().await{
                Ok(c) => c,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            };
            let result = client.execute("INSERT INTO files (user_id, filename, file_path) VALUES ($1, $2, $3)", &[&user_id, &filename, &path]).await;
            match result {
                Ok(_) => {},
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
            return (StatusCode::OK, format!("Uploaded as {}", filename)).into_response();
        }
    }
    (StatusCode::BAD_REQUEST, "No file found in request".to_string()).into_response()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect()
}

async fn get_user(State(pool): State<Pool>,
                  Json(payload): Json<QueryUser>) -> impl IntoResponse{
        let client = match pool.get().await {
            Ok(c) => c,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        };
        
        let row = match client.query_opt("SELECT id, username FROM users WHERE id = $1", &[&payload.user_id]).await{
            Ok(Some(row)) => row,
            Ok(None) => return (StatusCode::NOT_FOUND, "User not found".to_string()).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        };

        let user = User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            containers: Vec::new(), // TODO: Load actual containers
        };

        Json(user).into_response()
    }

async fn create_user(State(pool): State<Pool>,
                     Json(payload): Json<CreateUser>
                    ) -> impl IntoResponse
    {
        let client = match pool.get().await {
            Ok(c) => c,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
        let check = client.query("SELECT * FROM public.users WHERE username = $1", &[&payload.username]).await;

        if let Ok(value) = check {
            if value.len() != 0{
                return (StatusCode::CONFLICT,
                "Username is already in use!".to_string()).into_response();
        }
        }

        let insert_result = client
        .execute(
            "INSERT INTO users (username) VALUES ($1)", 
        &[&payload.username]
    ).await;

    match insert_result {
        Ok(_) => (StatusCode::CREATED, "User created".to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
    }

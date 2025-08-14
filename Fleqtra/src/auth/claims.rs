
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: i32,
    pub exp: usize,
    pub iat: usize,
    pub email: String,
    pub username: String,
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User{
    pub id: i32,
    pub email: String,
    pub username: String,
    pub containers: Vec<String>
}
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use crate::auth::claims::Claims;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct JwtError {
    pub message: String,
}

impl From<jsonwebtoken::errors::Error> for JwtError{
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        JwtError { message: format!("JWT error: {}", err) }
    }
}

const SECRET_KEY: &str = "1234567890"; //Load this from env variable

pub fn create_jwt(user_id: i32, email: String, username: String) -> Result<String, JwtError>{
    let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
    let claims = Claims {
        id: user_id,
        exp: now + 60*60*24,
        iat: now,
        email,
        username, 
    };

    let header = Header::default();
    let key = EncodingKey::from_secret(SECRET_KEY.as_ref());

    encode(&header, &claims, &key).map_err(JwtError::from)
}

pub fn validate_jwt(token: &str) -> Result<Claims, JwtError>{
    let key = DecodingKey::from_secret(SECRET_KEY.as_ref());
    let validation = Validation::default();

    decode(token, &key, &validation).map(|data| data.claims).map_err(JwtError::from)
}


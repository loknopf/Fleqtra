use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, Error, Salt};
use rand::{rngs::OsRng, TryRngCore};

pub fn hash_password(password: &str) -> Result<String, Error> {
    let mut bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    OsRng.try_fill_bytes(&mut bytes).unwrap();
    let salt = SaltString::encode_b64(&bytes).unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
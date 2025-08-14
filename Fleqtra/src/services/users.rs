use core::fmt;
use std::{error::Error, process::id};
use serde::de::value;
use tokio_postgres::{error::DbError, row, Error as PgError};

use deadpool_postgres::{Pool, PoolError};

use crate::{models::user::User, utils::user::verify_password};

#[derive(Clone)]
pub struct UserService{
    pool: Pool
}

#[derive(Debug)]
pub enum UserDatabaseError{
    DatabaseError(PgError),
    PoolError(PoolError),
    ValidationError(String),
    AlreadyExists(String),
    NotFound(String)
}

impl fmt::Display for UserDatabaseError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            UserDatabaseError::DatabaseError(error) => write!(f, "Database error: {}", error),
            UserDatabaseError::PoolError(pool_error) => write!(f, "Connection pool error: {}", pool_error),
            UserDatabaseError::ValidationError(reason) => write!(f, "Validation error: {}", reason),
            UserDatabaseError::AlreadyExists(message) => write!(f, "AlreadyExistsError: {}", message),
            UserDatabaseError::NotFound(message) => write!(f, "NotFoundError: {}", message),
        }
    }
}

impl Error for UserDatabaseError{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UserDatabaseError::DatabaseError(error) => Some(error),
            UserDatabaseError::PoolError(pool_error) => Some(pool_error),
            _ => None,
        }
    }
}

impl From<PgError> for UserDatabaseError{
    fn from(error: PgError) -> Self {
        UserDatabaseError::DatabaseError(error)
    }
}

impl From<PoolError> for UserDatabaseError{
    fn from(error: PoolError) -> Self {
        UserDatabaseError::PoolError(error)
    }
}


impl UserService{
    pub fn new(pool: Pool) -> Self{
        Self {pool}
    }

    pub async fn create_user(&self, username: String, email: String, password: String) -> Result<i32, UserDatabaseError>{
        let client = self.pool.get().await?;

        let row = client.query_one("INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id", 
                                        &[&username, &email, &password]
                                       )
                                       .await?;
        let id = row.get("id");
        Ok(id)
    }

    pub async fn delete_user(&self, id: &i32) -> Result<(), UserDatabaseError>{
        let client = self.pool.get().await?;
        let modified_rows = client.execute("DELETE FROM users WHERE id = $1", &[id]).await?;
        if modified_rows != 1 {
            Err(UserDatabaseError::NotFound("User could not be deleted, not found.".to_string()))
        }else {
            Ok(())            
        }
    }

    pub async fn update_user(&self, user: &User) -> Result<(), UserDatabaseError>{
        let client = self.pool.get().await?;
        client.execute("UPDATE users SET username = $1, email = $2 WHERE id = $3", &[&user.username, &user.email, &user.id])
                      .await?;
        Ok(())
    }

    pub async fn get_user_by_username(&self, username: &String) -> Result<User, UserDatabaseError>{
        let client = self.pool.get().await?;

        let row_opt = client.query_opt("SELECT id, email, username, containers FROM users WHERE username = $1", &[&username]).await?;
        
        match row_opt{
            Some(row) => {
                let user = User{ 
                    id: row.get("id"), 
                    email: row.get("email"), 
                    username: row.get("username"),  
                    containers: row.get("containers") 
                };
                Ok(user)
            },
            None => Err(UserDatabaseError::NotFound(format!("User with name '{}' not found", username)))
        }
    }

    pub async fn get_user_by_email(&self, email: &String) -> Result<User, UserDatabaseError>{
        let client = self.pool.get().await?;

        let row_opt = client.query_opt("SELECT id, email, username, containers FROM users WHERE email = $1", &[&email]).await?;
        
        match row_opt{
            Some(row) => {
                let user = User{ 
                    id: row.get("id"), 
                    email: row.get("email"), 
                    username: row.get("username"),  
                    containers: row.get("containers") 
                };
                Ok(user)
            },
            None => Err(UserDatabaseError::NotFound(format!("User with email '{}' not found", email)))
        }
    }

    pub async fn get_user_by_id(&self, id: &i32) -> Result<User, UserDatabaseError>{
        let client = self.pool.get().await?;

        let row_opt = client.query_opt("SELECT id, email, username, containers FROM users WHERE id = $1", &[&id]).await?;
        
        match row_opt{
            Some(row) => {
                let user = User{ 
                    id: row.get("id"), 
                    email: row.get("email"), 
                    username: row.get("username"),  
                    containers: row.get("containers") 
                };
                Ok(user)
            },
            None => Err(UserDatabaseError::NotFound(format!("User with id '{}' not found", id)))
        }
    }

    pub async fn get_user_password(&self, username: &String) -> Result<String, UserDatabaseError>{
        let client = self.pool.get().await?;
        
        let row_opt = client.query_opt("SELECT password FROM users WHERE username = $1", &[username]).await?;
        
        match row_opt{
            Some(row) => Ok(row.get("password")),
            None => Err(UserDatabaseError::NotFound(format!("User with username '{}' not found.", username)))
        }
    }   
}
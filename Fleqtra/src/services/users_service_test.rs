#[cfg(test)]
mod user_service_tests{
    use crate::{models::user::{self, User}, services::users::{UserDatabaseError, UserService}};

    use super::*;
    use axum::http::response;
    use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
    use tokio_postgres::NoTls;
    use std::{env, path::Path};

    fn load_env(){
        dotenvy::from_path(Path::new("./config/.env.test")).ok();
    }

    async fn create_test_db_pool() -> Pool{
        load_env();
        let mut cfg = Config::new();
        cfg.host = Some(env::var("TEST_DB_HOST")
                    .unwrap_or_else(|_| "localhost".to_string()));
        cfg.port = Some(env::var("TEST_DB_PORT")
                    .unwrap_or_else(|_| "5432".to_string())
                    .parse()
                    .expect("Invalid port number"));
        cfg.dbname = Some(env::var("TEST_DB_NAME").unwrap_or_else(|_| "fleqtra_test".to_string()));
        cfg.user = Some(env::var("TEST_DB_USER").unwrap_or_else(|_| "postgres".to_string()));
        cfg.password = Some(env::var("TEST_DB_PASSWORD").unwrap_or_else(|_| "password".to_string()));
        cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
        cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
            .expect("Failed to create a database pool")
    }

    async fn setup_test_tables(pool: &Pool){
        let client = pool.get().await.expect("Failed to get database connection");

        client.execute("CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        username VARCHAR(255) UNIQUE NOT NULL,
        email VARCHAR(255) UNIQUE NOT NULL,
        password VARCHAR(255) NOT NULL,
        containers TEXT[] DEFAULT '{}')", &[])
        .await
        .expect("Failed to create user table");
    }

    async fn cleanup_test_data(pool: &Pool){
        let client = pool.get().await.expect("Failed to get database connection");
        client.execute("DELETE FROM users", &[])
        .await
        .expect("Failed to cleanup test data");
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let pool = create_test_db_pool().await;
        setup_test_tables(&pool).await;
        let service = UserService::new(pool.clone());
        let result = service.create_user(
            "Smiley".to_string(),
            "testmail@bwb.net".to_string(), 
            "password123".to_string()).await;
        assert!(result.is_ok(), "Failed to create user: {:?}", result);
        let user_id = result.unwrap();
        assert!(user_id > 0, "User ID should be positive");
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_create_user_duplicate(){
        let pool = create_test_db_pool().await;
        setup_test_tables(&pool).await;
        let service = UserService::new(pool.clone());
        let result_1 = service.create_user(
            "Smiley".to_string(),
            "testmail@bwb.net".to_string(), 
            "password123".to_string()).await;
        assert!(result_1.is_ok(), "Failed to create user: {:?}", result_1);
        let user_id = result_1.unwrap();
        assert!(user_id > 0, "User ID should be positive");
        let result_2 = service.create_user(
            "Smiley".to_string(),
            "testmail2@bwb.net".to_string(), 
            "password123".to_string()).await;
        assert!(result_2.is_err(), "Should return error for duplicate username");
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_update_user(){
        let pool = create_test_db_pool().await;
        setup_test_tables(&pool).await;
        let service = UserService::new(pool.clone());
        let user_id_result = service.create_user("Smiley".to_string(), "smiley@e.com".to_string(), "password123".to_string()).await;
        assert!(user_id_result.is_ok(), "Failed to create user {:?}", user_id_result);
        let user_id = user_id_result.unwrap();
        assert!(user_id > 0, "User ID should be positive");
        let user_result = service.get_user_by_id(&user_id).await;
        assert!(user_result.is_ok());
        let mut user = user_result.unwrap();
        user.username = "NewSmiley".to_string();
        let update_result = service.update_user(&user).await;
        assert!(update_result.is_ok(), "Failed to update user {:?}", update_result);
        let new_user_result = service.get_user_by_id(&user_id).await;
        assert!(new_user_result.is_ok(), "Failed to get user from database {:?}", new_user_result);
        let new_user = new_user_result.unwrap();
        assert!(new_user.username == "NewSmiley".to_string(), "New username was not set correctly");
        cleanup_test_data(&pool).await;
    }


    #[tokio::test]
    async fn test_delete_user(){
        let pool = create_test_db_pool().await;
        setup_test_tables(&pool).await;
        let service = UserService::new(pool.clone());
        let user_id_result = service.create_user("Smiley".to_string(), "smiley@e.com".to_string(), "password123".to_string()).await;
        assert!(user_id_result.is_ok(), "Failed to create user {:?}", user_id_result);
        let user_id = user_id_result.unwrap();
        assert!(user_id > 0, "User ID should be positive");
        let delete_result = service.delete_user(&user_id).await;
        assert!(delete_result.is_ok(), "Failed to delete the user: {:?}", delete_result);
        let deleted_user_result = service.get_user_by_id(&user_id).await;
        match deleted_user_result{
            Ok(_) => panic!("Expected error when getting deleted user"),
            Err(UserDatabaseError::NotFound(msg)) => {
                assert!(msg.contains(&user_id.to_string()))
            }
            Err(other_err) => panic!("Expected NotFound error, got {:?}", other_err)
        }
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_user(){
        let pool = create_test_db_pool().await;
        setup_test_tables(&pool).await;
        let service = UserService::new(pool.clone());
        let user_id_result = service.create_user("Smiley".to_string(), "smiley@e.com".to_string(), "password123".to_string()).await;
        assert!(user_id_result.is_ok(), "Failed to create user {:?}", user_id_result);
        let user_id = user_id_result.unwrap();
        assert!(user_id > 0, "User ID should be positive");
        let user_by_id_result = service.get_user_by_id(&user_id).await;
        assert!(user_by_id_result.is_ok(), "Failed to get user by id {:?}", user_by_id_result );
        let user_by_id = user_by_id_result.unwrap();
        assert!(user_by_id.username == "Smiley".to_string(), "Username was not equal");
        let user_by_username_result = service.get_user_by_username(&"Smiley".to_string()).await;
        assert!(user_by_username_result.is_ok(), "Failed to get user by username {:?}", user_by_username_result);
        let user_by_username = user_by_username_result.unwrap();
        assert!(user_by_username.username == "Smiley".to_string(), "Username was not equal");
        let user_by_mail_result = service.get_user_by_email(&"smiley@e.com".to_string()).await;
        assert!(user_by_mail_result.is_ok(), "Failed to get user by mail {:?}", user_by_mail_result);
        let user_by_mail = user_by_mail_result.unwrap();
        assert!(user_by_mail.username == "Smiley".to_string(), "Username was not equal");
        cleanup_test_data(&pool).await
    }
}
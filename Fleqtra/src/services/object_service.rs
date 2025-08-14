use crate::utils::object::Object;
use std::path::{Path, PathBuf};
use std::{fs, io};
use deadpool_postgres::{Pool, PoolError};
use serde::de::value;
use tokio::fs as tokio_fs;
use tokio_postgres::Error as PgError;
use uuid::{Error, Uuid};
use log::{info, warn, error, debug};

pub enum ObjectServiceError{
    UuidError(Error),
    IOError(io::Error),
    DatabaseError(PgError),
    PoolError(PoolError),
    NotFoundError(String),
    PathError(String),
}

impl From<PgError> for ObjectServiceError{
    fn from(value: PgError) -> Self {
        ObjectServiceError::DatabaseError(value)
    }
}

impl From<PoolError> for ObjectServiceError{
    fn from(value: PoolError) -> Self {
        ObjectServiceError::PoolError(value)
    }
}

impl From<uuid::Error> for ObjectServiceError{
    fn from(value: uuid::Error) -> Self {
        Self::UuidError(value)
    }
}

impl From<io::Error> for ObjectServiceError{
    fn from(value: io::Error) -> Self {
        ObjectServiceError::IOError(value)
    }
}

pub struct ObjectService{
    pool: Pool
}

impl ObjectService {
    pub async fn get_object(&self, uuid: &Uuid) -> Result<Object, ObjectServiceError>{
        let client = self.pool.get().await?;
        let row_opt = client.query_opt("SELECT uuid, name, container_id, path FROM objects WHERE uuid = $1 ", &[&uuid.to_string()]).await?;
        let obj = match row_opt{
            Some(row) => {
                let uuid = Uuid::parse_str(row.get("uuid"))?;
                let container_uuid = Uuid::parse_str(row.get("container_id"))?;
                let path_string: String = row.get("path");
                let path = PathBuf::from(path_string);
                Object::new(uuid, row.get("name"), container_uuid, path)
            },
            None => return Err(ObjectServiceError::NotFoundError(format!("No object found for {}.", uuid.to_string()))),
        };

        Ok(obj)
    }
    ///Gets the content of a stored object from the local disk
    /// 
    /// # Arguments
    /// 
    /// * `obj` - ref to Object struct
    /// 
    /// # Example
    /// ```rust
    /// let content_result = service.get_object_storage(&obj).await.unwrap();
    /// ```
    pub async fn get_object_storage(&self, obj: &Object) -> io::Result<Vec<u8>>{
        debug!("Retrieving object {} ({}) from {}", obj.uuid(), obj.name(), obj.path().to_str().unwrap_or_else(|| "<invalid utf-8 path>"));
        tokio_fs::read(obj.path()).await
    }

    pub async fn store_object(&self, obj: &Object, content: &[u8]) -> Result<(), ObjectServiceError>{
        debug!("Storing object {} ({}) in {}", obj.uuid(), obj.name(), obj.path().to_str().unwrap_or_else(|| "<invalid utf-8 path>"));
        let path_str = match obj.path().to_str(){
            Some(value) => value,
            None => return Err(ObjectServiceError::PathError(format!("Could not parse path."))),
        };
        tokio_fs::create_dir_all(obj.path()).await?;
        tokio_fs::write(obj.path(), content).await?;
        debug!("Creating database entry for object");
        let client = self.pool.get().await?;
        client.execute("INSERT INTO objects (uuid, name, container_id, path);", 
                       &[&obj.uuid().to_string(), 
                                obj.name(), 
                                &obj.container_id().to_string(), 
                                &path_str])
                                .await?;
        Ok(())
    }

    pub async fn delte_object(&self, obj: &Object) -> Result<(), ObjectServiceError>{
        debug!("Removing object {} ({})", obj.uuid(), obj.name());
        tokio_fs::remove_file(obj.path()).await?;
        debug!("Removing object from database");
        let client = self.pool.get().await?;
        client.execute("DELETE FROM objects WHERE uuid = $1", &[&obj.uuid().to_string()]).await?;
        debug!("Searching parent directories for files");
        let mut obj_parent = obj.path().parent();
        while let Some(parent) = obj_parent{
            let mut dir_content = tokio_fs::read_dir(parent).await?;
            let is_empty = dir_content.next_entry().await?.is_none();
            debug!("Directory {} is {}", parent.to_str().unwrap_or_else(||"<invalid utf-8 path>"), if is_empty {"empty"} else {"not empty"});
            if is_empty {
                debug!("Removing empty directory");
                tokio_fs::remove_dir(parent).await?;
                obj_parent = parent.parent();
            } else {
                debug!("Directory not empty, not removing it");
                break;
            }
        }
        Ok(())
    }
}

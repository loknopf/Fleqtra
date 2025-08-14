use std::ops::SubAssign;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use tokio::fs as tokio_fs;

pub struct StorageService{
    base_path: PathBuf
    //compression -> future feature
}

impl StorageService{
    
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self{
        Self { base_path: base_path.as_ref().to_path_buf() }
    }

    pub async fn store_file(&self, sub_path: &str, content: &[u8]) -> io::Result<()>{
        let full_path = self.base_path.join(sub_path);
        if let Some(parent) = full_path.parent() {
            tokio_fs::create_dir_all(parent).await?;
        }
        tokio_fs::write(full_path, content).await
    }

    pub async fn get_file(&self, sub_path: &str) -> io::Result<Vec<u8>>{
        let full_path = self.base_path.join(sub_path);
        tokio_fs::read(full_path).await
    }

    pub async fn file_exists(&self, sub_path: &str) -> io::Result<bool>{
        let full_path = self.base_path.join(sub_path);
        tokio_fs::try_exists(full_path).await
    }

    pub async fn remove_file(&self, sub_path: &str) -> io::Result<()>{
        let full_path = self.base_path.join(sub_path);
        tokio_fs::remove_file(full_path).await
    }

    pub async fn rename_file(&self, old_sub_path: &str, new_sub_path: &str) -> io::Result<()>{
        let full_path = self.base_path.join(old_sub_path);
        let new_full_path = self.base_path.join(new_sub_path);
        tokio_fs::rename(full_path, new_full_path).await
    }
}
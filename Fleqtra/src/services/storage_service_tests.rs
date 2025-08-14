mod store_service_test{
    use std::path::{self, Path};

    use crate::services::storage::StorageService;
    #[tokio::test]
    async fn test_save_remove_file(){
        let test_content: Vec<u8> = vec![1,2,3,4,5];
        let service = StorageService::new(Path::new("test/storage/"));
        let store_result = service.store_file("file.txt", &test_content).await;
        assert!(store_result.is_ok());
        let remove_result = service.remove_file("file.txt").await;
        assert!(remove_result.is_ok());
    }

    #[tokio::test]
    async fn test_save_rename_file(){
        let test_content: Vec<u8> = vec![1,2,3,4,5];
        let service = StorageService::new(Path::new("test/storage/"));
        let store_result = service.store_file("file.txt", &test_content).await;
        assert!(store_result.is_ok());
        let rename_result = service.rename_file("file.txt", "file2.txt").await;
        assert!(rename_result.is_ok());
        let remove_result = service.remove_file("file.txt").await;
        assert!(remove_result.is_err());
        let remove_result = service.remove_file("file2.txt").await;
        assert!(remove_result.is_ok());
    }
}
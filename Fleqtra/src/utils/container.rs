use uuid::Uuid;

pub struct Container{
    container_id: Uuid,
    name: String,
    files: i64,
    //rules:Rules,
}
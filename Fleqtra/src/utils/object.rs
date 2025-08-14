use std::path::PathBuf;

use uuid::Uuid;

pub struct Object {
    id: Uuid,
    name: String,
    container_id: Uuid,
    path: PathBuf,
}

impl Object{
    pub fn new(id: Uuid, name: String, container_id: Uuid, path: PathBuf) -> Self{
        Self {id: id, name: name, container_id: container_id, path: path}
    }

    pub fn path(&self) -> &PathBuf{
        &self.path
    }

    pub fn name(&self) -> &String{
        &self.name
    }

    pub fn uuid(&self) -> &Uuid{
        &self.id
    }

    pub fn container_id(&self) -> &Uuid{
        &self.container_id
    }
}

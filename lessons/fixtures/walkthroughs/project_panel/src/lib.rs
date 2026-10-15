pub mod utils;

pub struct ServiceWorker {
    pub node_id: String,
    pub active: bool,
}

impl ServiceWorker {
    pub fn new(id: &str) -> Self {
        Self {
            node_id: id.to_string(),
            active: true,
        }
    }

    pub fn execute(&self) -> String {
        format!("Worker {} completed dispatch", self.node_id)
    }
}

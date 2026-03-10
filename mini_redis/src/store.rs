use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type du store partagé entre les clients
pub type Store = Arc<Mutex<HashMap<String, String>>>;

/// Crée un nouveau store vide
pub fn new_store() -> Store {
    Arc::new(Mutex::new(HashMap::new()))
}

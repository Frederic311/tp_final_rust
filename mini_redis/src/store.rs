use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Entrée dans le store avec valeur et expiration optionnelle
#[derive(Clone, Debug)]
pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

impl Entry {
    /// Crée une nouvelle entrée sans expiration
    pub fn new(value: String) -> Self {
        Entry {
            value,
            expires_at: None,
        }
    }

    /// Crée une nouvelle entrée avec expiration
    pub fn with_expiration(value: String, expires_at: Instant) -> Self {
        Entry {
            value,
            expires_at: Some(expires_at),
        }
    }

    /// Vérifie si l'entrée a expiré
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() >= expires_at
        } else {
            false
        }
    }
}

/// Type du store partagé entre les clients
pub type Store = Arc<Mutex<HashMap<String, Entry>>>;

/// Crée un nouveau store vide
pub fn new_store() -> Store {
    Arc::new(Mutex::new(HashMap::new()))
}

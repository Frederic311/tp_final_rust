use crate::protocol::{Request, Response};
use crate::store::Store;
use std::time::{Duration, Instant};

/// KEYS - Liste toutes les clés (non expirées)
pub async fn handle_keys(store: &Store) -> Response {
    let store = store.lock().await;
    let keys: Vec<String> = store
        .iter()
        .filter(|(_, entry)| !entry.is_expired())
        .map(|(key, _)| key.clone())
        .collect();
    Response::ok_with_keys(keys)
}

/// EXPIRE - Définit une expiration sur une clé
pub async fn handle_expire(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };
    let seconds = match request.seconds {
        Some(s) => s,
        None => return Response::error("missing seconds"),
    };

    let mut store = store.lock().await;
    if let Some(entry) = store.get_mut(&key) {
        if !entry.is_expired() {
            let expires_at = Instant::now() + Duration::from_secs(seconds);
            entry.expires_at = Some(expires_at);
            Response::ok()
        } else {
            Response::error("key not found")
        }
    } else {
        Response::error("key not found")
    }
}

/// TTL - Retourne le temps restant avant expiration
pub async fn handle_ttl(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let store = store.lock().await;
    match store.get(&key) {
        Some(entry) => {
            if entry.is_expired() {
                // Clé expirée = considérée comme inexistante
                Response::ok_with_ttl(-2)
            } else if let Some(expires_at) = entry.expires_at {
                // Clé avec expiration : calculer le temps restant
                let now = Instant::now();
                if expires_at > now {
                    let remaining = expires_at.duration_since(now);
                    Response::ok_with_ttl(remaining.as_secs() as i64)
                } else {
                    Response::ok_with_ttl(-2)
                }
            } else {
                // Clé sans expiration
                Response::ok_with_ttl(-1)
            }
        }
        None => {
            // Clé inexistante
            Response::ok_with_ttl(-2)
        }
    }
}

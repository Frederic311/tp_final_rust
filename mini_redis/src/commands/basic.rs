use crate::protocol::{Request, Response};
use crate::store::{Entry, Store};

/// PING - Test de connexion
pub fn handle_ping() -> Response {
    Response::ok()
}

/// SET - Stocke une paire clé/valeur
pub async fn handle_set(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };
    let value = match request.value {
        Some(v) => v,
        None => return Response::error("missing value"),
    };

    let mut store = store.lock().await;
    store.insert(key, Entry::new(value));
    Response::ok()
}

/// GET - Récupère la valeur associée à une clé
pub async fn handle_get(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let store = store.lock().await;
    let value = store.get(&key).and_then(|entry| {
        if entry.is_expired() {
            None
        } else {
            Some(entry.value.clone())
        }
    });
    Response::ok_with_value(value)
}

/// DEL - Supprime une clé
pub async fn handle_del(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let mut store = store.lock().await;
    let count = if store.remove(&key).is_some() { 1 } else { 0 };
    Response::ok_with_count(count)
}

use crate::protocol::{Request, Response};
use crate::store::{Entry, Store};
use std::time::{Duration, Instant};

/// Traite une requête et retourne la réponse appropriée
pub async fn process_request(request: Request, store: &Store) -> Response {
    let cmd = request.cmd.to_uppercase();

    match cmd.as_str() {
        "PING" => handle_ping(),
        "SET" => handle_set(request, store).await,
        "GET" => handle_get(request, store).await,
        "DEL" => handle_del(request, store).await,
        "KEYS" => handle_keys(store).await,
        "EXPIRE" => handle_expire(request, store).await,
        _ => Response::error("unknown command"),
    }
}

/// PING - Test de connexion
fn handle_ping() -> Response {
    Response::ok()
}

/// SET - Stocke une paire clé/valeur
async fn handle_set(request: Request, store: &Store) -> Response {
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
async fn handle_get(request: Request, store: &Store) -> Response {
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
async fn handle_del(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let mut store = store.lock().await;
    let count = if store.remove(&key).is_some() { 1 } else { 0 };
    Response::ok_with_count(count)
}

/// KEYS - Liste toutes les clés (non expirées)
async fn handle_keys(store: &Store) -> Response {
    let store = store.lock().await;
    let keys: Vec<String> = store
        .iter()
        .filter(|(_, entry)| !entry.is_expired())
        .map(|(key, _)| key.clone())
        .collect();
    Response::ok_with_keys(keys)
}

/// EXPIRE - Définit une expiration sur une clé
async fn handle_expire(request: Request, store: &Store) -> Response {
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

/// Nettoie les clés expirées du store (appelé par la tâche de fond)
pub async fn cleanup_expired_keys(store: &Store) {
    let mut store = store.lock().await;
    store.retain(|_, entry| !entry.is_expired());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::new_store;

    #[tokio::test]
    async fn test_ping() {
        let store = new_store();
        let request = Request {
            cmd: "PING".to_string(),
            key: None,
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[tokio::test]
    async fn test_set_get() {
        let store = new_store();

        // SET
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("hello".to_string()),
            value: Some("world".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        // GET
        let request = Request {
            cmd: "GET".to_string(),
            key: Some("hello".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"world\""));
    }

    #[tokio::test]
    async fn test_del() {
        let store = new_store();

        // SET puis DEL
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("test".to_string()),
            value: Some("value".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        let request = Request {
            cmd: "DEL".to_string(),
            key: Some("test".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":1"));

        // DEL une clé inexistante
        let request = Request {
            cmd: "DEL".to_string(),
            key: Some("nonexistent".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":0"));
    }

    #[tokio::test]
    async fn test_unknown_command() {
        let store = new_store();
        let request = Request {
            cmd: "INVALID".to_string(),
            key: None,
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"error\""));
        assert!(json.contains("unknown command"));
    }

    #[tokio::test]
    async fn test_keys() {
        let store = new_store();

        // Store vide
        let request = Request {
            cmd: "KEYS".to_string(),
            key: None,
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"keys\":[]"));

        // Ajouter des clés
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("key1".to_string()),
            value: Some("value1".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        let request = Request {
            cmd: "SET".to_string(),
            key: Some("key2".to_string()),
            value: Some("value2".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        // KEYS doit retourner les 2 clés
        let request = Request {
            cmd: "KEYS".to_string(),
            key: None,
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("key1"));
        assert!(json.contains("key2"));
    }

    #[tokio::test]
    async fn test_expire() {
        let store = new_store();

        // Créer une clé
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("temp".to_string()),
            value: Some("value".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        // Définir une expiration de 2 secondes
        let request = Request {
            cmd: "EXPIRE".to_string(),
            key: Some("temp".to_string()),
            value: None,
            seconds: Some(2),
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));

        // La clé doit encore exister
        let request = Request {
            cmd: "GET".to_string(),
            key: Some("temp".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"value\""));

        // Attendre l'expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // La clé doit avoir expiré
        let request = Request {
            cmd: "GET".to_string(),
            key: Some("temp".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"value\":null"));
    }
}

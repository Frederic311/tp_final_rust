use crate::protocol::{Request, Response};
use crate::store::Store;

/// Traite une requête et retourne la réponse appropriée
pub async fn process_request(request: Request, store: &Store) -> Response {
    let cmd = request.cmd.to_uppercase();

    match cmd.as_str() {
        "PING" => handle_ping(),
        "SET" => handle_set(request, store).await,
        "GET" => handle_get(request, store).await,
        "DEL" => handle_del(request, store).await,
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
    store.insert(key, value);
    Response::ok()
}

/// GET - Récupère la valeur associée à une clé
async fn handle_get(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let store = store.lock().await;
    let value = store.get(&key).cloned();
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
}

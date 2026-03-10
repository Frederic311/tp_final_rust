mod advanced;
mod basic;
mod expiration;

use crate::protocol::{Request, Response};
use crate::store::Store;

// Ré-exporter les fonctions publiques
pub use advanced::{handle_decr, handle_incr, handle_save};
pub use basic::{handle_del, handle_get, handle_ping, handle_set};
pub use expiration::{handle_expire, handle_keys, handle_ttl};

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
        "TTL" => handle_ttl(request, store).await,
        "INCR" => handle_incr(request, store).await,
        "DECR" => handle_decr(request, store).await,
        "SAVE" => handle_save(store).await,
        _ => Response::error("unknown command"),
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

    #[tokio::test]
    async fn test_ttl() {
        let store = new_store();

        // TTL sur clé inexistante → -2
        let request = Request {
            cmd: "TTL".to_string(),
            key: Some("nonexistent".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ttl\":-2"));

        // Créer une clé sans expiration
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("permanent".to_string()),
            value: Some("value".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        // TTL sur clé sans expiration → -1
        let request = Request {
            cmd: "TTL".to_string(),
            key: Some("permanent".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ttl\":-1"));

        // Créer une clé avec expiration de 10 secondes
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("expiring".to_string()),
            value: Some("value".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        let request = Request {
            cmd: "EXPIRE".to_string(),
            key: Some("expiring".to_string()),
            value: None,
            seconds: Some(10),
        };
        process_request(request, &store).await;

        // TTL doit être entre 8 et 10 secondes
        let request = Request {
            cmd: "TTL".to_string(),
            key: Some("expiring".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ttl\":"));
        // Vérifier que c'est un nombre positif
        assert!(!json.contains("\"ttl\":-"));
    }

    #[tokio::test]
    async fn test_incr() {
        let store = new_store();

        // INCR sur clé inexistante → crée avec valeur 1
        let request = Request {
            cmd: "INCR".to_string(),
            key: Some("counter".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"value\":1"));

        // INCR sur clé existante → incrémente
        let request = Request {
            cmd: "INCR".to_string(),
            key: Some("counter".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"value\":2"));

        // SET une valeur non-entière puis INCR → erreur
        let request = Request {
            cmd: "SET".to_string(),
            key: Some("notint".to_string()),
            value: Some("hello".to_string()),
            seconds: None,
        };
        process_request(request, &store).await;

        let request = Request {
            cmd: "INCR".to_string(),
            key: Some("notint".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"error\""));
        assert!(json.contains("not an integer"));
    }

    #[tokio::test]
    async fn test_decr() {
        let store = new_store();

        // DECR sur clé inexistante → crée avec valeur -1
        let request = Request {
            cmd: "DECR".to_string(),
            key: Some("counter".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"value\":-1"));

        // DECR sur clé existante → décrémente
        let request = Request {
            cmd: "DECR".to_string(),
            key: Some("counter".to_string()),
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"value\":-2"));
    }

    #[tokio::test]
    async fn test_save() {
        let store = new_store();

        // Ajouter des données
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

        // SAVE
        let request = Request {
            cmd: "SAVE".to_string(),
            key: None,
            value: None,
            seconds: None,
        };
        let response = process_request(request, &store).await;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
    }
}

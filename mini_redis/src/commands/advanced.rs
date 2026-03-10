use crate::protocol::{Request, Response};
use crate::store::{Entry, Store};
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// INCR - Incrémente une valeur entière
pub async fn handle_incr(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let mut store = store.lock().await;

    // Récupérer ou créer la valeur
    let current_value = if let Some(entry) = store.get(&key) {
        if entry.is_expired() {
            // Clé expirée, on la traite comme inexistante
            0
        } else {
            // Parser la valeur existante
            match entry.value.parse::<i64>() {
                Ok(v) => v,
                Err(_) => return Response::error("not an integer"),
            }
        }
    } else {
        // Clé inexistante, commence à 0
        0
    };

    let new_value = current_value + 1;
    store.insert(key, Entry::new(new_value.to_string()));
    Response::ok_with_int_value(new_value)
}

/// DECR - Décrémente une valeur entière
pub async fn handle_decr(request: Request, store: &Store) -> Response {
    let key = match request.key {
        Some(k) => k,
        None => return Response::error("missing key"),
    };

    let mut store = store.lock().await;

    // Récupérer ou créer la valeur
    let current_value = if let Some(entry) = store.get(&key) {
        if entry.is_expired() {
            // Clé expirée, on la traite comme inexistante
            0
        } else {
            // Parser la valeur existante
            match entry.value.parse::<i64>() {
                Ok(v) => v,
                Err(_) => return Response::error("not an integer"),
            }
        }
    } else {
        // Clé inexistante, commence à 0
        0
    };

    let new_value = current_value - 1;
    store.insert(key, Entry::new(new_value.to_string()));
    Response::ok_with_int_value(new_value)
}

/// SAVE - Sauvegarde le store dans dump.json
pub async fn handle_save(store: &Store) -> Response {
    let store = store.lock().await;

    // Créer un HashMap avec seulement les clés non expirées
    let mut data = HashMap::new();
    for (key, entry) in store.iter() {
        if !entry.is_expired() {
            data.insert(key.clone(), entry.value.clone());
        }
    }

    // Sérialiser en JSON
    let json = match serde_json::to_string_pretty(&data) {
        Ok(j) => j,
        Err(e) => return Response::error(format!("serialization error: {}", e)),
    };

    // Écrire dans le fichier
    match File::create("dump.json").await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(json.as_bytes()).await {
                return Response::error(format!("write error: {}", e));
            }
            Response::ok()
        }
        Err(e) => Response::error(format!("file error: {}", e)),
    }
}

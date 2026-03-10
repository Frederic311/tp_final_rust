mod commands;
mod handler;
mod protocol;
mod store;

use tokio::net::TcpListener;
use tokio::time::{interval, Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialiser tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Créer le store partagé
    let store = store::new_store();

    // Bind le listener sur le port 7878
    let addr = "127.0.0.1:7878";
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    info!("MiniRedis server listening on {}", addr);

    // Lancer la tâche de nettoyage des clés expirées
    let cleanup_store = store.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            commands::cleanup_expired_keys(&cleanup_store).await;
        }
    });

    // Accept loop : accepter les connexions et spawner une tâche par client
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New client connected: {}", addr);
                let store = store.clone();
                tokio::spawn(async move {
                    if let Err(e) = handler::handle_client(socket, store).await {
                        error!("Error handling client {}: {}", addr, e);
                    }
                    info!("Client disconnected: {}", addr);
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

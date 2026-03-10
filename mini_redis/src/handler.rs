use crate::commands::process_request;
use crate::protocol::{Request, Response};
use crate::store::Store;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::warn;

/// Gère une connexion client
pub async fn handle_client(
    socket: TcpStream,
    store: Store,
) -> Result<(), Box<dyn std::error::Error>> {
    let (read_half, mut write_half) = socket.into_split();
    let reader = BufReader::new(read_half);
    let mut lines = reader.lines();

    // Lire les commandes ligne par ligne
    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parser la requête JSON
        let response = match serde_json::from_str::<Request>(line) {
            Ok(request) => process_request(request, &store).await,
            Err(e) => {
                warn!("Invalid JSON: {} - Error: {}", line, e);
                Response::error("invalid json")
            }
        };

        // Envoyer la réponse en JSON + newline
        let response_json = serde_json::to_string(&response)?;
        write_half.write_all(response_json.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;
    }

    Ok(())
}

use tokio::net::TcpListener;
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::sync::mpsc;
use std::io;
use shared;
use log::{info, warn};

#[tokio::main]
async fn main() -> io::Result<()> {
    shared::logger::init().unwrap();

    let args = shared::Args::parse_args();

    let config: shared::ServerConfig = shared::config::parse_server_config(&args.config);

    let tunnel_listener = TcpListener::bind(&config.server.bind_addr).await?;
    let public_listener = TcpListener::bind("0.0.0.0:443").await?;

    info!("Relay listening on {} (agent) and 443 (public)", &config.server.bind_addr);

    let (tx, mut rx) = mpsc::channel::<tokio::net::TcpStream>(1);

    // Accept agent connection
    tokio::spawn(async move {
        loop {
            let (stream, addr) = tunnel_listener.accept().await.unwrap();
            info!("Agent connected: {}", addr);
            tx.send(stream).await.unwrap();
        }
    });

    loop {
        let (mut inbound, addr) = public_listener.accept().await?;
        info!("Incoming client: {}", addr);

        let mut agent = match rx.recv().await {
            Some(s) => s,
            None => {
                warn!("No agent available");
                continue;
            }
        };

        tokio::spawn(async move {
            info!("Forwarding to agent");

            if agent.write_all(b"START\n").await.is_err() {
                warn!("Failed to signal agent");
                return;
            }

            let _ = copy_bidirectional(&mut inbound, &mut agent).await;
        });
    }
}

use tokio::net::TcpStream;
use tokio::io::{copy_bidirectional, AsyncReadExt};
use tokio::net::TcpStream as TokioTcpStream;
use std::io;
use shared;
use log::{info};

#[tokio::main]
async fn main() -> io::Result<()> {
    shared::logger::init().unwrap();

    let args = shared::Args::parse_args();

    let config: shared::ClientConfig = shared::config::parse_client_config(&args.config);

    loop {
        info!("Connecting to relay...");
        let mut tunnel = TcpStream::connect(&config.client.remote_addr).await?;

        let mut buffer = [0u8; 6];

        loop {
            if tunnel.read_exact(&mut buffer).await.is_err() {
                info!("Tunnel closed");
                break;
            }

            if &buffer == b"START\n" {
                let mut local = TokioTcpStream::connect(&config.client.endpoint_addr).await?;
                let _ = copy_bidirectional(&mut tunnel, &mut local).await;
                break;
            }
        }
    }
}

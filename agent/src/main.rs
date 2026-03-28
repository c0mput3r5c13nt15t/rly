use tokio::net::TcpStream;
use tokio::io::{copy_bidirectional, AsyncReadExt};
use tokio::net::TcpStream as TokioTcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let relay_addr = "relay:7000";

    loop {
        println!("Connecting to relay...");
        let mut tunnel = TcpStream::connect(relay_addr).await?;

        let mut buffer = [0u8; 6];

        loop {
            if tunnel.read_exact(&mut buffer).await.is_err() {
                println!("Tunnel closed");
                break;
            }

            if &buffer == b"START\n" {
                let mut local = TokioTcpStream::connect("web:8080").await?;
                let _ = copy_bidirectional(&mut tunnel, &mut local).await;
                break;
            }
        }
    }
}

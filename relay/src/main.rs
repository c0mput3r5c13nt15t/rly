use tokio::net::TcpListener;
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tunnel_listener = TcpListener::bind("0.0.0.0:7000").await?;
    let public_listener = TcpListener::bind("0.0.0.0:443").await?;

    println!("Relay listening on 7000 (agent) and 443 (public)");

    let (tx, mut rx) = mpsc::channel::<tokio::net::TcpStream>(1);

    // Accept agent connection
    tokio::spawn(async move {
        loop {
            let (stream, _) = tunnel_listener.accept().await.unwrap();
            println!("Agent connected");
            tx.send(stream).await.unwrap();
        }
    });

    loop {
        let (mut inbound, addr) = public_listener.accept().await?;
        println!("Incoming client: {}", addr);

        // 👇 receive agent HERE (not inside spawn)
        let mut agent = match rx.recv().await {
            Some(s) => s,
            None => {
                println!("No agent available");
                continue;
            }
        };

        tokio::spawn(async move {
            println!("Forwarding to agent");

            if agent.write_all(b"START\n").await.is_err() {
                println!("Failed to signal agent");
                return;
            }

            let _ = copy_bidirectional(&mut inbound, &mut agent).await;
        });
    }
}

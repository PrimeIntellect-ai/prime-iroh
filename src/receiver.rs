use anyhow::{Context, Result};
use iroh::{Endpoint, endpoint::{Connection, RecvStream}};

const ALPN: &[u8] = b"hello-world";

pub struct IrohReceiver {
    endpoint: Endpoint,
    connection: Connection,
    recv_stream: RecvStream,
}

impl IrohReceiver {
    pub async fn new() -> Result<Self> {
        println!("Receiver starting...");
        let endpoint = Endpoint::builder()
            .discovery_n0()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;
        println!("cargo run --bin sender {}", endpoint.node_id());
        let connection = endpoint.accept().await.context("no incoming connection")?.await?;
        let recv_stream = connection.accept_uni().await?;

        Ok(Self { endpoint, connection, recv_stream })
    }

    pub async fn recv(&mut self) -> Result<()> {
        let mut size = [0; 4];
        self.recv_stream.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size) as usize;
        let mut msg = vec![0; size];
        self.recv_stream.read_exact(&mut msg).await?;
        println!("Received {} bytes", size);
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Receiver closing...");
        self.connection.closed().await;
        self.endpoint.close().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut receiver = IrohReceiver::new().await?;

    for _ in 0..10 {
        let _msg = receiver.recv().await?;
    }

    receiver.close().await?;

    Ok(())
}
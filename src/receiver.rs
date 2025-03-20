use anyhow::{Context, Result};
use iroh::{Endpoint, endpoint::Connection};

const ALPN: &[u8] = b"hello-world";

pub struct IrohReceiver {
    endpoint: Endpoint,
    connection: Connection,
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

        Ok(Self { endpoint, connection })
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        let mut recv_stream = self.connection.accept_uni().await?;
        let msg = recv_stream.read_to_end(usize::MAX).await?;
        Ok(msg)
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Receiver closing...");
        self.endpoint.close().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut receiver = IrohReceiver::new().await?;

    for _ in 0..100 {
        let msg = receiver.recv().await?;
        println!("Received: {}", String::from_utf8(msg)?);
    }

    receiver.close().await?;

    Ok(())
}
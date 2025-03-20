use anyhow::{Context, Result};
use iroh::Endpoint;

const ALPN: &[u8] = b"hello-world";

pub struct IrohReceiver {
    endpoint: Endpoint,
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

        Ok(Self { endpoint })
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        println!("Receiver waiting for message...");
        let connection = self.endpoint.accept().await.context("no incoming connection")?.await?;
        let mut recv_stream = connection.accept_uni().await?;
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

    for _ in 0..10 {
        let msg = receiver.recv().await?;
        println!("Received: {}", String::from_utf8(msg)?);
    }

    receiver.close().await?;

    Ok(())
}
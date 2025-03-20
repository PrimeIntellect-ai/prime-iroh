use anyhow::{Context, Result};
use iroh::{Endpoint, endpoint::{Connection, RecvStream}};
use std::sync::Arc;
use tokio::sync::Mutex;

const ALPN: &[u8] = b"hello-world";

#[derive(Clone)]
pub struct IrohReceiver {
    endpoint: Endpoint,
    connection: Connection,
    recv_streams: Vec<Arc<Mutex<RecvStream>>>,
}

impl IrohReceiver {
    pub async fn new(num_streams: usize) -> Result<Self> {
        println!("Receiver starting...");
        let endpoint = Endpoint::builder()
            .discovery_n0()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;
        println!("cargo run --bin sender {}", endpoint.node_id());
        let connection = endpoint.accept().await.context("no incoming connection")?.await?;
        let mut recv_streams = Vec::with_capacity(num_streams);
        for _ in 0..num_streams {
            let recv_stream = connection.accept_uni().await?;
            recv_streams.push(Arc::new(Mutex::new(recv_stream)));
        }

        Ok(Self { endpoint, connection, recv_streams })
    }

    pub async fn recv(&mut self, tag: usize) -> Result<Vec<u8>> {
        // Lock the receiving stream
        let stream = &mut self.recv_streams[tag];
        let mut stream = stream.lock().await;

        // Read the size of the message
        let mut size = [0; 4];
        stream.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size) as usize;

        // Read the message
        let mut msg = vec![0; size];
        stream.read_exact(&mut msg).await?;

        Ok(msg)
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Receiver closing...");
        for stream in self.recv_streams.iter_mut() {
            let mut stream = stream.lock().await;
            stream.stop(0u32.into())?;
        }
        self.connection.closed().await;
        self.endpoint.close().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup
    let num_new_tokens = 10;
    let num_micro_batches = 4;

    // Create receiver
    let mut receiver = IrohReceiver::new(num_micro_batches).await?;

    // Receive messages
    for _ in 0..num_new_tokens {
        for micro_batch_idx in 0..num_micro_batches {
            let mut receiver_clone = receiver.clone();
            let msg = receiver_clone.recv(micro_batch_idx).await;
            println!("Received msg: {:?}", String::from_utf8(msg.unwrap()).unwrap());
        }
    }

    // Close the receiver
    receiver.close().await?;

    Ok(())
}
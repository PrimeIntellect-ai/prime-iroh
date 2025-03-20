use anyhow::{Context, Result};
use iroh::{Endpoint, endpoint::{Connection, RecvStream}};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

const ALPN: &[u8] = b"hello-world";

pub struct IrohReceiver {
    runtime: Runtime,
    endpoint: Endpoint,
    connection: Connection,
    recv_streams: Vec<Arc<Mutex<RecvStream>>>,
}

impl IrohReceiver {
    pub fn new(num_streams: usize) -> Result<Self> {
        let runtime = Runtime::new()?;
        let (endpoint, connection, recv_streams) = runtime.block_on(async {
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
            Ok::<(Endpoint, Connection, Vec<Arc<Mutex<RecvStream>>>), anyhow::Error>((endpoint, connection, recv_streams))
        })?;

        Ok(Self { runtime, endpoint, connection, recv_streams })
    }
    
    pub fn irecv(&mut self, tag: usize) -> JoinHandle<Result<Vec<u8>>> {
        let stream = self.recv_streams[tag].clone();
        self.runtime.spawn(async move {
            let mut stream = stream.lock().await;
            // Read the size of the message
            let mut size = [0; 4];
            stream.read_exact(&mut size).await?;
            let size = u32::from_le_bytes(size) as usize;

            // Read the message
            let mut msg = vec![0; size];
            stream.read_exact(&mut msg).await?;
            Ok(msg)
        })
    }

    pub fn recv(&mut self, tag: usize) -> Result<Vec<u8>> {
        let handle = self.irecv(tag);
        self.runtime.block_on(handle)?
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            for stream in self.recv_streams.iter_mut() {
                let mut stream = stream.lock().await;
                stream.stop(0u32.into())?;
            }
            self.connection.closed().await;
            self.endpoint.close().await;
            Ok(())
        })
    }
}

fn main() -> Result<()> {
    // Setup
    let num_new_tokens = 4;
    let num_micro_batches = 2;

    // Create receiver
    let mut receiver = IrohReceiver::new(num_micro_batches)?;

    // Receive messages
    for _ in 0..num_new_tokens {
        for micro_batch_idx in 0..num_micro_batches {
            let msg = receiver.recv(micro_batch_idx)?;
            println!("Received msg: {:?}", String::from_utf8(msg).unwrap());
        }
    }

    // Close the receiver
    receiver.close()?;

    Ok(())
}
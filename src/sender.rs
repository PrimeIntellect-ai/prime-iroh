use anyhow::{Error, Result};
use iroh::{
    Endpoint, NodeAddr, NodeId,
    endpoint::{Connection, SendStream},
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::work::SendWork;

const ALPN: &[u8] = b"hello-world";

pub struct SenderConnection {
    connection: Connection,
    send_streams: Vec<Arc<Mutex<SendStream>>>,
}

impl SenderConnection {
    pub fn new(connection: Connection, send_streams: Vec<Arc<Mutex<SendStream>>>) -> Self {
        Self {
            connection,
            send_streams,
        }
    }
}

pub struct Sender {
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    connection: Option<SenderConnection>,
}

impl Sender {
    pub fn new(runtime: Arc<Runtime>, endpoint: Endpoint) -> Self {
        Self {
            runtime,
            endpoint,
            connection: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.connection.is_some()
    }

    pub fn connect(
        &mut self,
        peer_id_str: String,
        num_streams: usize,
        num_retries: usize,
        backoff_ms: usize,
    ) -> Result<()> {
        let mut retries_left = num_retries;
        let mut backoff = tokio::time::Duration::from_millis(backoff_ms as u64);

        while retries_left > 0 {
            match self.runtime.block_on(async {
                // Get node address from node_id
                let addr = self.get_node_addr(node_id_str)?;

                // Try to establish connection
                let connection = self.endpoint.connect(addr, ALPN).await?;

                // Make sure the connection is established by sending dummy payload (0u32)
                let mut send_streams = Vec::with_capacity(num_streams);
                for _ in 0..num_streams {
                    let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
                    send_stream
                        .lock()
                        .await
                        .write_all(&(0u32.to_le_bytes()))
                        .await?;
                    send_streams.push(send_stream);
                }

                // Create sender connection struct
                let sender_connection = SenderConnection::new(connection, send_streams);

                Ok::<SenderConnection, Error>(sender_connection)
            }) {
                Ok(sender_connection) => {
                    self.connection = Some(sender_connection);
                    return Ok(());
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to connect to node after {} retries: {}",
                        num_retries, e
                    );
                    println!("{}", msg);
                    retries_left -= 1;
                    if retries_left == 0 {
                        return Err(anyhow::anyhow!(msg));
                    }

                    // Wait with exponential backoff before retrying
                    println!("Waiting for {}ms before retrying", backoff.as_millis());
                    self.runtime.block_on(async {
                        tokio::time::sleep(backoff).await;
                    });

                    // Exponential backoff
                    backoff *= 2;
                }
            }
        }
        unreachable!()
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize, latency: Option<usize>) -> SendWork {
        let stream = self.connection.as_ref().unwrap().send_streams[tag].clone();
        let handle = self.runtime.spawn(async move {
            if let Some(latency) = latency {
                tokio::time::sleep(tokio::time::Duration::from_millis(latency as u64)).await;
            }
            let mut stream = stream.lock().await;
            let size = msg.len() as u32;
            stream.write_all(&size.to_le_bytes()).await?;
            stream.write_all(&msg).await?;

            Ok(())
        });
        SendWork {
            runtime: self.runtime.clone(),
            handle: handle,
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self.runtime.block_on(async {
            let mut connection = self.connection.take();
            if let Some(connection) = connection.as_mut() {
                // First flush all streams
                for stream in connection.send_streams.iter() {
                    let mut stream = stream.lock().await;
                    stream.finish()?; // Make sure all data is sent
                    stream.stopped().await?;
                }

                // Then close the connection
                connection.connection.close(0u32.into(), b"close");

                // Wait a moment for the close to propagate
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Finally close the endpoint
            self.endpoint.close().await;
            Ok::<(), Error>(())
        }) {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("Failed to close sender with error: {}", e); // TODO: WARNING
                Ok(())
            }
        }
    }

    fn get_node_addr(&self, node_id_str: &str) -> Result<NodeAddr> {
        let bytes = hex::decode(node_id_str)?;
        let node_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
        Ok(NodeAddr::new(node_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_creation() -> Result<()> {
        let runtime = Arc::new(Runtime::new()?);
        let endpoint =
            runtime.block_on(async { Endpoint::builder().discovery_n0().bind().await })?;

        let sender = Sender::new(runtime, endpoint);
        assert!(!sender.is_ready());

        Ok(())
    }
}

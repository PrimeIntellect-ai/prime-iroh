use anyhow::Result;
use iroh::{NodeId, NodeAddr, Endpoint, endpoint::{Connection, SendStream}};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;

use crate::work::Work;

const ALPN: &[u8] = b"hello-world";

pub struct SenderConnection {
    connection: Connection,
    send_streams: Vec<Arc<Mutex<SendStream>>>,
}

impl SenderConnection {
    pub fn new(connection: Connection, send_streams: Vec<Arc<Mutex<SendStream>>>) -> Self {
        Self { connection, send_streams }
    }
}

pub struct Sender {
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    connection: Option<SenderConnection>,
}

impl Sender {
    pub fn new(runtime: Arc<Runtime>, endpoint: Endpoint) -> Self {
        Self { runtime, endpoint, connection: None }
    }

    pub fn is_ready(&self) -> bool {
        self.connection.is_some()
    }

    pub fn connect(&mut self, node_id_str: &str, num_streams: usize) -> Result<()> {
        let connection = self.runtime.block_on(async {
            let addr = self.get_node_addr(node_id_str)?;
            let connection = self.endpoint.connect(addr, ALPN).await?;
            let mut send_streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
                send_stream.lock().await.write_all(&(0u32.to_le_bytes())).await?;
                send_streams.push(send_stream);
            }
            Ok::<SenderConnection, anyhow::Error>(SenderConnection::new(connection, send_streams))
        })?;
        self.connection = Some(connection);
        Ok(())
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize) -> Work<()> {
        let stream = self.connection.as_ref().unwrap().send_streams[tag].clone();
        let handle = self.runtime.spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let mut stream = stream.lock().await;
            let size = msg.len() as u32;
            stream.write_all(&size.to_le_bytes()).await?;
            stream.write_all(&msg).await?;
            Ok(())
        });
        Work {
            runtime: self.runtime.clone(),
            handle,
        }
    }

    pub fn _send(&mut self, msg: Vec<u8>, tag: usize) -> Result<()> {
        self.isend(msg, tag).wait()
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            if let Some(connection) = self.connection.as_mut() {
                // First flush all streams
                for stream in connection.send_streams.iter() {
                    let mut stream = stream.lock().await;
                    stream.finish()?;  // Make sure all data is sent
                    stream.stopped().await?;
                }
                
                // Then gracefully close the connection
                connection.connection.close(0u32.into(), b"close");
                
                // Wait a moment for the close to propagate
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            // Finally close the endpoint
            self.endpoint.close().await;
            Ok(())
        })
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
        let endpoint = runtime.block_on(async {
            Endpoint::builder().discovery_n0().bind().await
        })?;
        
        let sender = Sender::new(runtime, endpoint);
        assert!(!sender.is_ready());
        
        Ok(())
    }
}
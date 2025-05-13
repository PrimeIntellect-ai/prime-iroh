use anyhow::{Error, Result, anyhow, ensure};
use iroh::{
    Endpoint, NodeAddr, NodeId,
    endpoint::{Connection, SendStream},
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::work::SendWork;

const ALPN: &[u8] = b"prime-iroh";

pub struct MultiStreamConnection {
    connection: Connection,
    send_streams: Vec<Arc<Mutex<SendStream>>>,
}

impl MultiStreamConnection {
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
    connection: Option<MultiStreamConnection>,
}

impl Sender {
    pub fn new(runtime: Arc<Runtime>, endpoint: Endpoint) -> Self {
        log::info!("Creating sender (ID={})", endpoint.node_id().fmt_short());
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
    ) -> Result<()> {
        // Ensure we don't already have a connection
        ensure!(self.connection.is_none(), "Already have a connection");

        // Get the peer address from the node id
        let peer_addr = self.get_node_addr(peer_id_str.clone())?;

        log::info!(
            "Connecting {}->{}",
            self.endpoint.node_id().fmt_short(),
            peer_addr.node_id.fmt_short()
        );

        // Connection loop
        let mut retries_left = num_retries;
        while retries_left > 0 {
            match self.runtime.block_on(async {
                // Try to establish connection
                let connection = self.endpoint.connect(peer_addr.clone(), ALPN).await?;

                // Establish streams by sending dummy payload
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

                Ok::<MultiStreamConnection, Error>(MultiStreamConnection::new(
                    connection,
                    send_streams,
                ))
            }) {
                Ok(connection) => {
                    log::info!(
                        "Connected {}->{}",
                        self.endpoint.node_id().fmt_short(),
                        connection.connection.remote_node_id()?.fmt_short()
                    );
                    self.connection = Some(connection);
                    return Ok(());
                }
                Err(e) => {
                    retries_left -= 1;
                    let msg = format!(
                        "Failed to connect {}->{} after {} tries (left: {}): {}",
                        self.endpoint.node_id().fmt_short(),
                        peer_addr.node_id.fmt_short(),
                        num_retries - retries_left,
                        retries_left,
                        e
                    );
                    log::warn!("{}", msg);
                    if retries_left == 0 {
                        return Err(anyhow!(msg));
                    }
                }
            }
        }
        unreachable!()
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize, latency: Option<usize>) -> Result<SendWork> {
        // Ensure we have a connection
        ensure!(self.is_ready(), "Sender is not ready");
        log::debug!("Sending {} bytes via stream {}", msg.len(), tag);

        // Get the sender connection
        let connection = self.connection.as_ref().unwrap();

        // Get the stream
        ensure!(tag < connection.send_streams.len(), "Invalid tag");
        let stream = connection.send_streams[tag].clone();

        let handle = self.runtime.spawn(async move {
            if let Some(latency) = latency {
                tokio::time::sleep(tokio::time::Duration::from_millis(latency as u64)).await;
            }
            // Lock the stream
            let mut stream = stream.lock().await;

            // Write the size of the message
            let size = msg.len() as u32;
            stream.write_all(&size.to_le_bytes()).await?;

            // Write the message
            stream.write_all(&msg).await?;

            Ok(())
        });
        Ok(SendWork {
            runtime: self.runtime.clone(),
            handle: handle,
        })
    }

    pub fn close(&mut self) -> Result<()> {
        if !self.is_ready() {
            log::warn!("Sender connection does not exist, skipping close");
            return Ok(());
        }
        log::info!(
            "Closing sender (ID={})",
            self.endpoint.node_id().fmt_short()
        );
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
                log::warn!("Failed to close sender with error: {}", e);
                Ok(())
            }
        }
    }

    fn get_node_addr(&self, node_id_str: String) -> Result<NodeAddr> {
        let bytes = hex::decode(node_id_str)?;
        let node_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
        Ok(NodeAddr::new(node_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() -> (Endpoint, Arc<Runtime>) {
        let runtime = Arc::new(Runtime::new().unwrap());
        let endpoint = runtime
            .block_on(async { Endpoint::builder().discovery_n0().bind().await })
            .unwrap();
        (endpoint, runtime)
    }

    #[test]
    fn test_sender_creation() -> Result<()> {
        let (endpoint, runtime) = init();
        let sender = Sender::new(runtime, endpoint);
        assert!(!sender.is_ready());

        Ok(())
    }

    #[test]
    fn test_sender_error_on_send() -> Result<()> {
        let (endpoint, runtime) = init();
        let mut sender = Sender::new(runtime, endpoint);

        let res = sender.isend(vec![0; 100], 0, None);
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn test_sender_ok_on_close() -> Result<()> {
        let (endpoint, runtime) = init();
        let mut sender = Sender::new(runtime, endpoint);

        let res = sender.close();
        assert!(res.is_ok());

        Ok(())
    }
}

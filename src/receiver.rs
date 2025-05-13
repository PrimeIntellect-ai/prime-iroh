use anyhow::{Error, Result, ensure};
use iroh::protocol::{ProtocolHandler, Router};
use iroh::{
    Endpoint,
    endpoint::{Connection, RecvStream},
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::work::RecvWork;

const ALPN: &[u8] = b"prime-iroh";

#[derive(Clone, Debug)]
struct MultiStreamConnection {
    connection: Connection,
    recv_streams: Vec<Arc<Mutex<RecvStream>>>,
}

#[derive(Clone, Debug)]
struct ReceiverHandler {
    connection: Arc<Mutex<Option<MultiStreamConnection>>>,
    num_streams: usize,
}

impl ReceiverHandler {
    fn new(num_streams: usize, connection: Arc<Mutex<Option<MultiStreamConnection>>>) -> Self {
        Self {
            connection,
            num_streams,
        }
    }
}

impl ProtocolHandler for ReceiverHandler {
    fn accept(
        &self,
        conn: Connection,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> {
        let num_streams = self.num_streams;
        let connection = self.connection.clone();
        Box::pin(async move {
            let mut connection = connection.lock().await;
            ensure!(connection.is_none(), "Already have a connection");

            // Initialize receive streams
            let mut streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let mut recv_stream = conn.accept_uni().await?;
                let mut buffer = [0; 4]; // Buffer to hold the 0u32 value
                recv_stream.read_exact(&mut buffer).await?;
                streams.push(Arc::new(Mutex::new(recv_stream)));
            }

            // Store connection and streams
            let connection_ref = MultiStreamConnection {
                connection: conn,
                recv_streams: streams,
            };
            *connection = Some(connection_ref);

            Ok(())
        })
    }
}

pub struct Receiver {
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    router: Router,
    connection: Arc<Mutex<Option<MultiStreamConnection>>>,
}

impl Receiver {
    pub fn new(runtime: Arc<Runtime>, endpoint: Endpoint, num_streams: usize) -> Self {
        log::info!("Creating receiver (ID={})", endpoint.node_id().fmt_short());
        let connection = Arc::new(Mutex::new(None));
        let handler = ReceiverHandler::new(num_streams, connection.clone());
        let router = runtime.block_on(async {
            Router::builder(endpoint.clone())
                .accept(ALPN, handler)
                .spawn()
                .await
                .expect("Failed to spawn router")
        });

        Self {
            runtime,
            endpoint,
            router,
            connection,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.runtime.block_on(async {
            let connection = self.connection.lock().await;
            connection.is_some()
        })
    }

    pub fn irecv(&mut self, tag: usize) -> Result<RecvWork> {
        // Ensure we have a connection
        ensure!(self.is_ready(), "Receiver is not ready");
        log::debug!("Receiving message via stream {}", tag);

        let connection = self.connection.clone();
        let handle = self.runtime.spawn(async move {
            // Get the optional state
            let connection = connection.lock().await;
            ensure!(connection.is_some(), "No connection available");

            // Unwrap the receiver state
            let connection = connection.as_ref().unwrap();
            ensure!(tag < connection.recv_streams.len(), "Invalid tag");

            // Get the stream
            let stream = connection.recv_streams[tag].clone();
            let mut stream = stream.lock().await;

            // Read the size of the message
            let mut size = [0; 4];
            stream.read_exact(&mut size).await?;
            let size = u32::from_le_bytes(size) as usize;

            // Read the message
            let mut msg = vec![0; size];
            stream.read_exact(&mut msg).await?;

            Ok(msg)
        });
        Ok(RecvWork {
            runtime: self.runtime.clone(),
            handle: handle,
        })
    }

    pub fn close(&mut self) -> Result<()> {
        if !self.is_ready() {
            log::warn!("Receiver connection does not exist, skipping close");
            return Ok(());
        }
        log::info!(
            "Closing receiver (ID={})",
            self.endpoint.node_id().fmt_short()
        );
        match self.runtime.block_on(async {
            let mut connection = self.connection.lock().await;

            if let Some(connection) = connection.take() {
                // Close receive streams if they exist
                for stream in &connection.recv_streams {
                    let mut stream = stream.lock().await;
                    stream.stop(0u32.into())?;
                }

                // Close connection if it exists
                connection.connection.closed().await;
            }

            // Shutdown router
            self.router.shutdown().await?;

            // Close endpoint
            self.endpoint.close().await;
            Ok::<(), Error>(())
        }) {
            Ok(()) => Ok(()),
            Err(e) => {
                log::warn!("Failed to close receiver with error: {}", e);
                Ok(())
            }
        }
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
    fn test_receiver_init() -> Result<()> {
        let (endpoint, runtime) = init();
        let receiver = Receiver::new(runtime, endpoint, 1);
        assert!(!receiver.is_ready());

        Ok(())
    }

    #[test]
    fn test_receiver_error_on_recv() -> Result<()> {
        let (endpoint, runtime) = init();
        let mut receiver = Receiver::new(runtime, endpoint, 1);

        let res = receiver.irecv(0);
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn test_receiver_ok_on_close() -> Result<()> {
        let (endpoint, runtime) = init();
        let mut receiver = Receiver::new(runtime, endpoint, 1);

        let res = receiver.close();
        assert!(res.is_ok());

        Ok(())
    }
}

use anyhow::Result;
use iroh::{Endpoint, endpoint::{Connection, RecvStream}};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;
use iroh::protocol::{ProtocolHandler, Router};

use crate::work::RecvWork;

const ALPN: &[u8] = b"hello-world";

#[derive(Clone, Debug)]
struct ReceiverState {
    connection: Connection,
    recv_streams: Vec<Arc<Mutex<RecvStream>>>,
}

#[derive(Clone, Debug)]
struct ReceiverHandler {
    state: Arc<Mutex<Option<ReceiverState>>>,
    num_streams: usize,
}

impl ReceiverHandler {
    fn new(num_streams: usize, state: Arc<Mutex<Option<ReceiverState>>>) -> Self {
        Self {
            state,
            num_streams,
        }
    }
}

impl ProtocolHandler for ReceiverHandler {
    fn accept(&self, conn: Connection) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
        let num_streams = self.num_streams;
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.lock().await;
            if state.is_some() {
                return Err(anyhow::anyhow!("Already have a connection"));
            }
            // Initialize receive streams
            let mut streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let mut recv_stream = conn.accept_uni().await?;
                let mut buffer = [0; 4]; // Buffer to hold the 0u32 value
                recv_stream.read_exact(&mut buffer).await?;
                streams.push(Arc::new(Mutex::new(recv_stream)));
            }

            // Store connection and streams
            let state_ref = ReceiverState {
                connection: conn,
                recv_streams: streams,
            };
            *state = Some(state_ref);
            
            Ok(())
        })
    }
}

pub struct Receiver {
    runtime: Arc<Runtime>,
    state: Arc<Mutex<Option<ReceiverState>>>,
    endpoint: Endpoint,
    router: Router,
}

impl Receiver {
    pub fn new(runtime: Arc<Runtime>, endpoint: Endpoint, num_streams: usize) -> Self {
        let state = Arc::new(Mutex::new(None));
        let handler = ReceiverHandler::new(num_streams, state.clone());
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
            state,
            router,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.runtime.block_on(async {
            let state = self.state.lock().await;
            state.is_some()
        })
    }
    
    pub fn irecv(&mut self, tag: usize) -> RecvWork {
        let state = self.state.clone();
        let handle = self.runtime.spawn(async move {
            let state = state.lock().await;
            
            if state.is_none() {
                return Err(anyhow::anyhow!("No receive streams available"));
            }
            
            let state = state.as_ref().unwrap();
            let stream = state.recv_streams[tag].clone();
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
        RecvWork {
            runtime: self.runtime.clone(),
            handle: handle,
        }
    }

    pub fn _recv(&mut self, tag: usize) -> Result<Vec<u8>> {
        let msg = self.irecv(tag).wait().unwrap();
        Ok(msg)
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            let state = self.state.lock().await;
            
            if let Some(state) = state.as_ref() {
                // Close receive streams if they exist
                for stream in &state.recv_streams {
                    let mut stream = stream.lock().await;
                    stream.stop(0u32.into())?;
                }
                
                // Close connection if it exists
                state.connection.closed().await;
            }

            // Shutdown router
            self.router.shutdown().await?;

            // Close endpoint
            self.endpoint.close().await;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_receiver_creation() -> Result<()> {
        let runtime = Arc::new(Runtime::new()?);
        let endpoint = runtime.block_on(async {
            Endpoint::builder().discovery_n0().bind().await
        })?;
        
        let receiver = Receiver::new(runtime, endpoint, 1);
        assert!(!receiver.is_ready());
        
        Ok(())
    }
}
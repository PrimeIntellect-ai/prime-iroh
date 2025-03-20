use anyhow::{Context, Result};
use iroh::{Endpoint, endpoint::{Connection, RecvStream}};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;
use iroh::protocol::{ProtocolHandler, Router};

use crate::work::Work;

const ALPN: &[u8] = b"hello-world";

#[derive(Clone)]
struct ReceiverState {
    connection: Option<Connection>,
    recv_streams: Option<Vec<Arc<Mutex<RecvStream>>>>,
}

#[derive(Clone)]
struct ReceiverHandler {
    state: Arc<Mutex<ReceiverState>>,
    num_streams: usize,
}

impl ReceiverHandler {
    fn new(num_streams: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(ReceiverState {
                connection: None,
                recv_streams: None,
            })),
            num_streams,
        }
    }
}

impl ProtocolHandler for ReceiverHandler {
    fn accept(&self, conn: Connection) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
        let state = self.state.clone();
        let num_streams = self.num_streams;
        
        Box::pin(async move {
            let mut state = state.lock().await;
            
            // Initialize receive streams
            let mut streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let recv_stream = conn.accept_uni().await?;
                streams.push(Arc::new(Mutex::new(recv_stream)));
            }

            // Store connection and streams
            state.connection = Some(conn);
            state.recv_streams = Some(streams);
            
            Ok(())
        })
    }
}

pub struct Receiver {
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    state: Arc<Mutex<ReceiverState>>,
    router: Option<Router>,
}

impl Receiver {
    pub fn new(num_streams: usize) -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        
        let endpoint = runtime.block_on(async {
            let endpoint = Endpoint::builder()
                .discovery_n0()
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await?;
            println!("cargo run sender {}", endpoint.node_id());
            Ok::<Endpoint, anyhow::Error>(endpoint)
        })?;

        let state = Arc::new(Mutex::new(ReceiverState {
            connection: None,
            recv_streams: None,
        }));

        let handler = ReceiverHandler::new(num_streams);

        // Set up router with protocol handler
        let router = runtime.block_on(async {
            Router::builder(endpoint.clone())
                .accept(ALPN, handler)
                .spawn()
                .await
        })?;

        Ok(Self {
            runtime,
            endpoint,
            state,
            router: Some(router),
        })
    }
    
    pub fn irecv(&mut self, tag: usize) -> Work<Vec<u8>> {
        let state = self.state.clone();
        let handle = self.runtime.spawn(async move {
            let state = state.lock().await;
            
            if state.recv_streams.is_none() {
                return Err(anyhow::anyhow!("No receive streams available"));
            }
            
            let stream = state.recv_streams.as_ref().unwrap()[tag].clone();
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
        Work {
            runtime: self.runtime.clone(),
            handle,
        }
    }

    pub fn recv(&mut self, tag: usize) -> Result<Vec<u8>> {
        self.irecv(tag).wait()
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            let state = self.state.lock().await;
            
            // Close receive streams if they exist
            if let Some(streams) = &state.recv_streams {
                for stream in streams {
                    let mut stream = stream.lock().await;
                    stream.stop(0u32.into())?;
                }
            }
            
            // Close connection if it exists
            if let Some(connection) = &state.connection {
                connection.closed().await;
            }
            
            self.endpoint.close().await;
            Ok(())
        })
    }
}

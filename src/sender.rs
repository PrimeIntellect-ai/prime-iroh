use anyhow::Result;
use iroh::{NodeAddr, Endpoint, endpoint::{Connection, SendStream}};
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

pub struct IrohSender {
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    connection: Option<SenderConnection>,
}

impl IrohSender {
    pub fn new() -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        let endpoint = runtime.block_on(async {
            let endpoint = Endpoint::builder().discovery_n0().bind().await?;
            Ok::<Endpoint, anyhow::Error>(endpoint)
        })?;
        Ok(Self { runtime, endpoint, connection: None })
    }

    pub fn connect(&mut self, addr: NodeAddr, num_streams: usize) -> Result<()> {
        let connection = self.runtime.block_on(async {
            let connection = self.endpoint.connect(addr, ALPN).await?;
            let mut send_streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
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
            for stream in self.connection.as_mut().unwrap().send_streams.iter() {
                let mut stream = stream.lock().await;
                stream.stopped().await?;
            }
            self.connection.as_ref().unwrap().connection.close(0u32.into(), b"close");
            self.endpoint.close().await;
            Ok(())
        })
    }
} 
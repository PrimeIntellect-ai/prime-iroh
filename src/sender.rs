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
    connection: SenderConnection,
}

impl IrohSender {
    pub fn new(addr: NodeAddr, num_streams: usize) -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        let (endpoint, connection) = runtime.block_on(async {
            let endpoint = Endpoint::builder().discovery_n0().bind().await?;
            let connection = endpoint.connect(addr, ALPN).await?;
            let mut send_streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
                send_streams.push(send_stream);
            }
            Ok::<(Endpoint, SenderConnection), anyhow::Error>((endpoint, SenderConnection::new(connection, send_streams)))
        })?;
        Ok(Self { runtime, endpoint, connection })
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize) -> Work<()> {
        let stream = self.connection.send_streams[tag].clone();
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

    pub fn send(&mut self, msg: Vec<u8>, tag: usize) -> Result<()> {
        self.isend(msg, tag).wait()
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            for stream in self.connection.send_streams.iter_mut() {
                let mut stream = stream.lock().await;
                stream.stopped().await?;
            }
            self.connection.connection.close(0u32.into(), b"close");
            self.endpoint.close().await;
            Ok(())
        })
    }
} 
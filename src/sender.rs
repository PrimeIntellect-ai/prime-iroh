use anyhow::Result;
use iroh::{NodeId, NodeAddr, Endpoint, endpoint::{Connection, SendStream}};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::runtime::Runtime;

const ALPN: &[u8] = b"hello-world";

pub struct SendHandle {
    runtime: Arc<Runtime>,
    handle: JoinHandle<Result<()>>,
}

impl SendHandle {
    pub fn wait(self) -> Result<()> {
        self.runtime.block_on(self.handle)?
    }
}

pub struct IrohSender {
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    connection: Connection,
    send_streams: Vec<Arc<Mutex<SendStream>>>,
}

impl IrohSender {
    pub fn new(addr: NodeAddr, num_streams: usize) -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        let (endpoint, connection, send_streams) = runtime.block_on(async {
            let endpoint = Endpoint::builder().discovery_n0().bind().await?;
            let connection = endpoint.connect(addr, ALPN).await?;
            let mut send_streams = Vec::with_capacity(num_streams);
            for _ in 0..num_streams {
                let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
                send_streams.push(send_stream);
            }
            Ok::<(Endpoint, Connection, Vec<Arc<Mutex<SendStream>>>), anyhow::Error>((endpoint, connection, send_streams))
        })?;
        Ok(Self { runtime, endpoint, connection, send_streams })
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize) -> SendHandle {
        let stream = self.send_streams[tag].clone();
        let handle = self.runtime.spawn(async move {
            let mut stream = stream.lock().await;
            let size = msg.len() as u32;
            stream.write_all(&size.to_le_bytes()).await?;
            stream.write_all(&msg).await?;
            Ok(())
        });
        SendHandle {
            runtime: self.runtime.clone(),
            handle,
        }
    }

    pub fn send(&mut self, msg: Vec<u8>, tag: usize) -> Result<()> {
        self.isend(msg, tag).wait()
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            for stream in self.send_streams.iter_mut() {
                let mut stream = stream.lock().await;
                stream.stopped().await?;
            }
            self.connection.close(0u32.into(), b"close");
            self.endpoint.close().await;
            Ok(())
        })
    }
} 

fn get_node_addr() -> Result<NodeAddr> {
    use std::env;

    let args: Vec<String> = env::args().collect();
    let node_id_str = args.get(1).expect("Expected node id as the first argument").clone();
    let bytes = hex::decode(node_id_str)?;
    let node_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
    Ok(NodeAddr::new(node_id))
}

fn main() -> Result<()> {
    // Setup
    let num_new_tokens = 4;
    let num_micro_batches = 2;

    // Create sender
    let addr = get_node_addr()?;
    let mut sender = IrohSender::new(addr, num_micro_batches)?;

    // Send messages
    for token_idx in 0..num_new_tokens {
        let mut handles = Vec::new();
        for micro_batch_idx in 0..num_micro_batches {
            // Send message
            println!("Sending token idx {} micro batch idx {}", token_idx, micro_batch_idx);
            let msg = vec![token_idx as u8; 1024 * 1024 * 50];
            let send_handle = sender.isend(msg, micro_batch_idx);
            handles.push(send_handle);
        }
        for handle in handles {
            let _ = handle.wait();
        }
    }


    // Close the sender
    let _ = sender.close();

    Ok(())
}
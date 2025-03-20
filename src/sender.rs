use anyhow::Result;
use iroh::{NodeId, NodeAddr, Endpoint, endpoint::{Connection, SendStream}};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const ALPN: &[u8] = b"hello-world";

#[derive(Clone)]
pub struct IrohSender {
    endpoint: Endpoint,
    connection: Connection,
    send_streams: Vec<Arc<Mutex<SendStream>>>,
}

impl IrohSender {
    pub async fn new(addr: NodeAddr, num_streams: usize) -> Result<Self> {
        println!("Sender starting...");
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr, ALPN).await?;
        let mut send_streams = Vec::with_capacity(num_streams);
        for _ in 0..num_streams {
            let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
            send_streams.push(send_stream);
        }

        Ok(Self { endpoint, connection, send_streams })
    }

    pub async fn send(&mut self, msg: Vec<u8>, tag: usize) -> Result<()> {
        let stream = &mut self.send_streams[tag];
        let mut stream = stream.lock().await;
        let size = msg.len() as u32;
        stream.write_all(&size.to_le_bytes()).await?;
        stream.write_all(&msg).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Sender closing...");
        for stream in self.send_streams.iter_mut() {
            let mut stream = stream.lock().await;
            stream.stopped().await?;
        }
        self.connection.close(0u32.into(), b"close");
        self.endpoint.close().await;
        Ok(())
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

#[tokio::main]
async fn main() -> Result<()> {
    // Setup
    let num_new_tokens = 10;
    let num_micro_batches = 4;

    // Create sender
    let addr = get_node_addr()?;
    let mut sender = IrohSender::new(addr, num_micro_batches).await?;

    // Send messages
    let mut handles = vec![];
    for token_idx in 0..num_new_tokens {
        for micro_batch_idx in 0..num_micro_batches {
            let mut sender_clone = sender.clone();  
            let handle = tokio::spawn(async move {
                let msg = format!("token_idx: {}, micro_batch_idx: {}", token_idx, micro_batch_idx);
                println!("Sending msg: {}", msg);
                // Pretend to do some work
                tokio::time::sleep(Duration::from_secs(1)).await;
                let _ = sender_clone.send(msg.as_bytes().to_vec(), micro_batch_idx).await;
            });
            handles.push(handle);
        }
        for handle in handles.iter_mut() {
            handle.await?;
        }
        handles.clear();
    }

    // Close the sender
    sender.close().await?;

    Ok(())
}
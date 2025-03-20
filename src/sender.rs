use anyhow::Result;
use iroh::{NodeId, NodeAddr, Endpoint};

const ALPN: &[u8] = b"hello-world";

pub struct IrohSender {
    endpoint: Endpoint,
}

impl IrohSender {
    pub async fn new() -> Result<Self> {
        println!("Sender starting...");
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;

        Ok(Self { endpoint })

    }

    pub async fn send(&self, addr: NodeAddr, msg: Vec<u8>) -> Result<()> {
        println!("Sender sending message...");
        let connection = self.endpoint.connect(addr, ALPN).await?;
        let mut send_stream = connection.open_uni().await?;
        send_stream.write_all(&msg).await?;
        send_stream.finish()?;
        send_stream.stopped().await?;
        connection.close(0u32.into(), b"bye!");
        println!("Sender signalized send done");
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Sender closing...");
        self.endpoint.close().await;
        Ok(())
    }
} 

#[tokio::main]
async fn main() -> Result<()> {
    use std::env;

    let mut sender = IrohSender::new().await?;

    let args: Vec<String> = env::args().collect();
    let node_id_str = args.get(1).expect("Expected node id as the first argument").clone();
    let bytes = hex::decode(node_id_str)?;
    let node_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
    let addr = NodeAddr::new(node_id);

    for _ in 0..10 {
        sender.send(addr.clone(), b"hello".to_vec()).await?;
    }

    sender.close().await?;

    Ok(())
}
use anyhow::Result;
use iroh::{NodeId, NodeAddr, Endpoint, endpoint::Connection};

const ALPN: &[u8] = b"hello-world";

pub struct IrohSender {
    endpoint: Endpoint,
    connection: Connection,
}

impl IrohSender {
    pub async fn new(addr: NodeAddr) -> Result<Self> {
        println!("Sender starting...");
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr, ALPN).await?;

        Ok(Self { endpoint, connection })
    }

    pub async fn send(&self, msg: Vec<u8>) -> Result<()> {
        let mut send_stream = self.connection.open_uni().await?;
        send_stream.write_all(&msg).await?;
        send_stream.finish()?;
        send_stream.stopped().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Sender closing...");
        self.connection.close(0u32.into(), b"bye!");
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
    let addr = get_node_addr()?;
    let mut sender = IrohSender::new(addr).await?;

    for num in 1..101 {
        sender.send(num.to_string().as_bytes().to_vec()).await?;
    }

    sender.close().await?;

    Ok(())
}
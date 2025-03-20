use anyhow::Result;
use iroh::{NodeId, NodeAddr, Endpoint, endpoint::{Connection, SendStream}};

const ALPN: &[u8] = b"hello-world";

pub struct IrohSender {
    endpoint: Endpoint,
    connection: Connection,
    send_stream: SendStream,
}

impl IrohSender {
    pub async fn new(addr: NodeAddr) -> Result<Self> {
        println!("Sender starting...");
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr, ALPN).await?;
        let send_stream = connection.open_uni().await?;

        Ok(Self { endpoint, connection, send_stream })
    }

    pub async fn send(&mut self, msg: Vec<u8>) -> Result<()> {
        let size = msg.len() as u32;
        self.send_stream.write_all(&size.to_le_bytes()).await?;
        self.send_stream.write_all(&msg).await?;
        println!("Sent {} bytes", size);
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        println!("Sender closing...");
        self.send_stream.stopped().await?;
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

    for num in 0..10 {
        let huge_array = vec![num; 1024 * 1024 * 50]; // 1MB array
        sender.send(huge_array).await?;
    }

    sender.close().await?;

    Ok(())
}
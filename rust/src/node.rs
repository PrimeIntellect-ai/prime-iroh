use crate::receiver::Receiver;
use crate::sender::Sender;
use crate::work::{RecvWork, SendWork};

use anyhow::{Error, Result};
use iroh::{Endpoint, SecretKey};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct Node {
    num_streams: usize,
    endpoint: Endpoint,
    receiver: Receiver,
    sender: Sender,
}

impl Node {
    pub fn new(num_streams: usize) -> Result<Self> {
        Self::with_seed(num_streams, None)
    }

    pub fn with_seed(num_streams: usize, seed: Option<u64>) -> Result<Self> {
        log::info!("Creating node");
        let runtime = Arc::new(Runtime::new()?);
        let endpoint = runtime.block_on(async {
            let mut builder = Endpoint::builder().discovery_n0();
            if let Some(seed) = seed {
                let mut rng = StdRng::seed_from_u64(seed);
                let secret_key = SecretKey::generate(&mut rng);
                builder = builder.secret_key(secret_key);
            }
            let endpoint = builder.bind().await?;
            Ok::<Endpoint, Error>(endpoint)
        })?;
        let receiver = Receiver::new(runtime.clone(), endpoint.clone(), num_streams);
        let sender = Sender::new(runtime.clone(), endpoint.clone());
        log::info!("Created node (ID={})", endpoint.node_id().fmt_short());
        Ok(Self {
            num_streams,
            endpoint,
            receiver,
            sender,
        })
    }

    pub fn node_id(&self) -> String {
        self.endpoint.node_id().to_string()
    }

    pub fn connect(&mut self, peer_id_str: String, num_retries: usize) -> Result<()> {
        self.sender
            .connect(peer_id_str, self.num_streams, num_retries)?;
        Ok(())
    }

    pub fn can_recv(&self) -> bool {
        self.receiver.is_ready()
    }

    pub fn can_send(&self) -> bool {
        self.sender.is_ready()
    }

    pub fn is_ready(&self) -> bool {
        self.can_recv() && self.can_send()
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize, latency: Option<usize>) -> Result<SendWork> {
        self.sender.isend(msg, tag, latency)
    }

    pub fn irecv(&mut self, tag: usize) -> Result<RecvWork> {
        self.receiver.irecv(tag)
    }

    pub fn close(&mut self) -> Result<()> {
        log::info!("Closing node (ID={})", self.endpoint.node_id().fmt_short());
        self.sender.close()?;
        self.receiver.close()?;
        log::info!("Closed node (ID={})", self.endpoint.node_id().fmt_short());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() -> Result<()> {
        let node = Node::new(1)?;
        assert!(node.node_id().len() == 64);
        assert!(!node.can_recv());
        assert!(!node.can_send());
        assert!(!node.is_ready());

        Ok(())
    }

    #[test]
    fn test_node_creation_with_seed() -> Result<()> {
        let node = Node::with_seed(1, Some(42))?;
        assert!(
            node.node_id() == "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454"
        );
        assert!(node.node_id().len() == 64);
        assert!(!node.can_recv());
        assert!(!node.can_send());
        assert!(!node.is_ready());

        Ok(())
    }
}

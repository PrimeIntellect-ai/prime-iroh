use crate::work::{SendWork, RecvWork};
use crate::sender::Sender;
use crate::receiver::Receiver;

use tokio::runtime::Runtime;
use std::sync::Arc;
use iroh::{Endpoint, SecretKey};
use anyhow::{Result, Error};
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct Node {
    node_id: String,
    num_micro_batches: usize,
    receiver: Receiver,
    sender: Sender,
}

impl Node {
    pub fn new(num_micro_batches: usize) -> Result<Self> {
        Self::with_seed(num_micro_batches, None)
    }

    pub fn with_seed(num_micro_batches: usize, seed: Option<u64>) -> Result<Self> {
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
        let node_id = endpoint.node_id().to_string();
        let receiver = Receiver::new(runtime.clone(), endpoint.clone(), num_micro_batches);
        let sender = Sender::new(runtime.clone(), endpoint.clone());
        Ok(Self { node_id, num_micro_batches, receiver, sender })
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn connect(&mut self, node_id_str: &str) -> Result<()> {
        self.sender.connect(node_id_str, self.num_micro_batches)?;
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

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize, latency: Option<usize>) -> SendWork {
        self.sender.isend(msg, tag, latency)
    }

    pub fn irecv(&mut self, tag: usize) -> RecvWork {
        self.receiver.irecv(tag)
    }

    pub fn close(&mut self) -> Result<()> {
        self.sender.close()?;
        self.receiver.close()?;
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
        assert!(node.node_id() == "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454");
        assert!(node.node_id().len() == 64);
        assert!(!node.can_recv());
        assert!(!node.can_send());
        assert!(!node.is_ready());

        Ok(())
    }
}

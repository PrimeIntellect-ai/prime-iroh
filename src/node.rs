use crate::work::Work;
use crate::sender::Sender;
use crate::receiver::Receiver;

use tokio::runtime::Runtime;
use std::sync::Arc;
use iroh::{Endpoint, NodeId, NodeAddr};
use anyhow::{Result, Error};

pub struct Node {
    pub node_id: NodeId,
    num_micro_batches: usize,
    receiver: Receiver,
    sender: Sender,
}

impl Node {
    pub fn new(num_micro_batches: usize) -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        let endpoint = runtime.block_on(async {
            let endpoint = Endpoint::builder().discovery_n0().bind().await?;
            Ok::<Endpoint, Error>(endpoint)
        })?;
        let node_id = endpoint.node_id();
        let receiver = Receiver::new(runtime.clone(), endpoint.clone(), num_micro_batches);
        let sender = Sender::new(runtime.clone(), endpoint.clone());
        Ok(Self { node_id, num_micro_batches, receiver, sender })
    }


    pub fn connect(&mut self, addr: NodeAddr) -> Result<()> {
        self.sender.connect(addr, self.num_micro_batches)?;
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

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize) -> Work<()> {
        self.sender.isend(msg, tag)
    }

    pub fn irecv(&mut self, tag: usize) -> Work<Vec<u8>> {
        self.receiver.irecv(tag)
    }

    pub fn close(&mut self) -> Result<()> {
        self.sender.close()?;
        self.receiver.close()?;
        Ok(())
    }
}

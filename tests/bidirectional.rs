use anyhow::Result;
use iroh_py::node::Node;
use std::time::Duration;

const NUM_MESSAGES: usize = 5;
const NUM_STREAMS: usize = 1;

struct BidirectionalTest {
    node0: Node,
    node1: Node,
}

impl BidirectionalTest {
    fn new() -> Result<Self> {
        // Initialize nodes
        let mut node0 = Node::with_seed(NUM_STREAMS, None)?;
        let mut node1 = Node::with_seed(NUM_STREAMS, None)?;
        
        // Wait for nodes to be ready
        std::thread::sleep(Duration::from_millis(1000));

        // Connect bidirectionally
        node0.connect(node1.node_id())?;
        node1.connect(node0.node_id())?;
        
        // Wait for connection to be established
        while !node0.can_recv() || !node1.can_send() {
            std::thread::sleep(Duration::from_millis(100));
        }
        
        Ok(Self { node0, node1 })
    }

    fn test_communication(&mut self) -> Result<()> {
        for i in 0..NUM_MESSAGES {
            self.node0.isend(format!("Message {} from node 0", i).as_bytes().to_vec(), 0, None);
            self.node1.isend(format!("Message {} from node 1", i).as_bytes().to_vec(), 0, None);

            let received_from_node0 = self.node1.irecv(0).wait()?;
            let received_from_node1 = self.node0.irecv(0).wait()?;

            assert_eq!(received_from_node0, format!("Message {} from node 0", i).as_bytes().to_vec());
            assert_eq!(received_from_node1, format!("Message {} from node 1", i).as_bytes().to_vec());
        }

        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_communication() -> Result<()> {
        let mut test = BidirectionalTest::new()?;
        
        // Test basic connection state
        assert!(test.node0.is_ready());
        assert!(test.node1.is_ready());
        assert!(test.node0.can_send());
        assert!(test.node1.can_send());
        assert!(test.node0.can_recv());
        assert!(test.node1.can_recv());

        // Test bidirectional communication
        test.test_communication()?;
        
        Ok(())
    }
}


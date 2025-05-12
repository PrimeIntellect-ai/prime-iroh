use anyhow::Result;
use prime_iroh::node::Node;

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
        println!("Initializing node 0 (ID: {})", node0.node_id());
        let mut node1 = Node::with_seed(NUM_STREAMS, None)?;

        // Currently needed for discovery to work
        // TODO(Mika): Fix this
        std::thread::sleep(Duration::from_millis(1000));

        // Connect bidirectionally
        node0.connect(node1.node_id())?;
        node1.connect(node0.node_id())?;

        // Wait for connection to be established
        while !node0.can_recv() || !node1.can_send() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Ok(Self { node0, node1 })
    }

    fn test_communication(&mut self) -> Result<()> {
        for i in 0..NUM_MESSAGES {
            let send_work0 = self.node0.isend(
                format!("Message {} from node 0", i).as_bytes().to_vec(),
                0,
                None,
            );
            let send_work1 = self.node1.isend(
                format!("Message {} from node 1", i).as_bytes().to_vec(),
                0,
                None,
            );

            send_work0.wait()?;
            send_work1.wait()?;

            let received_from_node0 = self.node1.irecv(0).wait()?;
            let received_from_node1 = self.node0.irecv(0).wait()?;

            assert_eq!(
                received_from_node0,
                format!("Message {} from node 0", i).as_bytes().to_vec()
            );
            assert_eq!(
                received_from_node1,
                format!("Message {} from node 1", i).as_bytes().to_vec()
            );
        }

        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        self.node0.close()?;
        self.node1.close()?;
        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_communication() -> Result<()> {
        let mut test = BidirectionalTest::new()?;

        // Test bidirectional communication
        test.test_communication()?;

        Ok(())
    }
}

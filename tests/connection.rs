use anyhow::Result;
use std::time::Duration;

use prime_iroh::node::Node;

const NUM_STREAMS: usize = 1;

struct ConnectionTest {
    nodes: Vec<Node>,
}

impl ConnectionTest {
    fn new(num_nodes: usize) -> Result<Self> {
        // Initialize nodes
        let mut nodes = Vec::new();
        let mut node_ids = Vec::new();
        for i in 0..num_nodes {
            let node = Node::new(NUM_STREAMS)?;
            let node_id = node.node_id();
            println!("Initializing node {} (ID: {})", i, node_id);
            nodes.push(node);
            node_ids.push(node_id);
        }

        // Connect nodes
        for i in 0..num_nodes {
            let current_node = &mut nodes[i];
            let j = (i + 1) % num_nodes;
            let node_id = current_node.node_id();
            let peer_id = node_ids[j].clone();
            println!(
                "Connecting node {}->{} (ID: {}->{})",
                i, j, node_id, peer_id
            );
            current_node.connect(peer_id, 10, 100)?;
        }

        while !nodes.iter().all(|node| node.is_ready()) {
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(Self { nodes })
    }

    // Helper method to verify connection state
    fn verify_active_connection_state(&self) -> Result<()> {
        for node in &self.nodes {
            assert!(node.is_ready(), "Node should be ready");
            assert!(node.can_send(), "Node should be able to send");
            assert!(node.can_recv(), "Node should be able to receive");
        }
        println!("All nodes are active");
        Ok(())
    }

    fn verify_inactive_connection_state(&self) -> Result<()> {
        for node in &self.nodes {
            assert!(!node.is_ready(), "Node should not be ready");
            assert!(!node.can_send(), "Node should not be able to send");
            assert!(!node.can_recv(), "Node should not be able to receive");
        }
        println!("All nodes are inactive");
        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            println!("Closing node {}", i);
            node.close()?;
        }
        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_connection() -> Result<()> {
        let num_nodes = 2;
        let mut test = ConnectionTest::new(num_nodes)?;

        // Test connection state
        test.verify_active_connection_state()?;

        // Teardown
        test.teardown()?;

        // Test connection state
        test.verify_inactive_connection_state()?;

        Ok(())
    }
}

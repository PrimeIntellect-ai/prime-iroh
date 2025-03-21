use anyhow::Result;
use iroh_py::node::Node;

fn run_node() -> Result<()> {
    let mut node = Node::with_seed(2, Some(42))?;
    let node_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454";
    node.connect(node_id)?;
    while !node.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(node.can_recv());
    assert!(node.can_send());
    assert!(node.is_ready());

    Ok(())
}

fn run_node_with_sync_message() -> Result<()> {
    let mut node = Node::with_seed(2, Some(42))?;
    let node_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454";
    node.connect(node_id)?;
    while !node.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let sent = b"Hello, world!".to_vec();
    node.isend(sent.clone(), 0).wait()?;
    let rcvd = node.irecv(0).wait()?;
    assert_eq!(sent, rcvd);
    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_connection() -> Result<()> {
        std::thread::spawn(run_node);
        std::thread::spawn(run_node);

        Ok(())
    }

    #[test]
    fn test_bidirectional_connection_with_message() -> Result<()> {
        std::thread::spawn(run_node_with_sync_message);
        std::thread::spawn(run_node_with_sync_message);

        Ok(())
    }
}


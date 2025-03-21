use anyhow::Result;
use iroh_py::node::Node;

fn run_receiver() -> Result<()> {
    let node = Node::with_seed(2, Some(42))?;
    while !node.can_recv() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Receiver ready");
    assert!(node.can_recv());
    assert!(!node.can_send());
    assert!(!node.is_ready());
    Ok(())
}

fn run_sender() -> Result<()> {
    let mut node = Node::new(2)?;
    let node_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454";
    node.connect(node_id)?;
    while !node.can_send() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Sender ready");

    assert!(node.can_send());
    assert!(!node.can_recv());
    assert!(!node.is_ready());

    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_node_connection() -> Result<()> {
        std::thread::spawn(run_receiver);
        std::thread::spawn(run_sender);

        Ok(())
    }
}


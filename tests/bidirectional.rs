use anyhow::Result;
use iroh_py::node::Node;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

fn connect_nodes() -> Result<()> {
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

fn send_single_message() -> Result<()> {
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

fn send_multiple_messages() -> Result<()> {
    let mut node = Node::with_seed(2, Some(42))?;
    let node_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454";
    node.connect(node_id)?;
    while !node.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let num_messages = 10;
    for i in 0..num_messages {
        let sent = format!("Message {}", i).as_bytes().to_vec();
        node.isend(sent.clone(), 0).wait()?;
        println!("Sent: {:?}", sent);
        let rcvd = node.irecv(0).wait()?;
        println!("Received: {:?}", rcvd);
        assert_eq!(sent, rcvd);
    }
    Ok(())
}

fn run_node(seed: u64, tag: &str) -> Result<()> {
    let mut node = Node::with_seed(2, Some(seed))?;
    let peer_id = if seed == 42 {
        // Connect to node with seed 43
        "da1d3d33264bebd2ff215473064fb11f4c7ceb4b820a3868c2c792d27e205691"
    } else {
        // Connect to node with seed 42
        "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454"
    };
    
    println!("{} connecting to {}", tag, peer_id);
    node.connect(peer_id)?;
    
    while !node.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("{} waiting for connection...", tag);
    }
    
    println!("{} connected and ready", tag);
    assert!(node.can_recv());
    assert!(node.can_send());
    
    // Send and receive multiple messages
    let num_messages = 10;
    for i in 0..num_messages {
        let sent = format!("Message {} from {}", i, tag).as_bytes().to_vec();
        println!("{} sending: {:?}", tag, std::str::from_utf8(&sent).unwrap());
        node.isend(sent.clone(), 0).wait()?;
        
        let rcvd = node.irecv(0).wait()?;
        println!("{} received: {:?}", tag, std::str::from_utf8(&rcvd).unwrap());
    }
    
    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_connection() -> Result<()> {
        // Start two nodes as subprocesses with different seeds
        let node1 = Command::new(std::env::current_exe()?)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .arg("run_node_42")
            .stdout(Stdio::piped())
            .spawn()?;
        
        let node2 = Command::new(std::env::current_exe()?)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .arg("run_node_43")
            .stdout(Stdio::piped())
            .spawn()?;
        
        // Monitor output
        let stdout1 = BufReader::new(node1.stdout.unwrap());
        let stdout2 = BufReader::new(node2.stdout.unwrap());
        
        for line in stdout1.lines() {
            println!("Node1: {}", line?);
        }
        
        for line in stdout2.lines() {
            println!("Node2: {}", line?);
        }

        Ok(())
    }
    
    #[test]
    fn run_node_42() -> Result<()> {
        run_node(42, "Node42")
    }
    
    #[test]
    fn run_node_43() -> Result<()> {
        run_node(43, "Node43")
    }
}


pub mod work;
mod sender;
mod receiver;
mod node;

use anyhow::Result;
use iroh::{NodeId, NodeAddr};

use node::Node;

fn get_node_addr() -> Result<NodeAddr> {
    use std::env;

    let args: Vec<String> = env::args().collect();
    let node_id_str = args.get(2).expect("Expected node id as the second argument").clone();
    let bytes = hex::decode(node_id_str)?;
    let node_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
    Ok(NodeAddr::new(node_id))
}

fn run_sender() -> Result<()> {
    // Setup
    let num_new_tokens = 4;
    let num_micro_batches = 2;

    // Create sender
    let mut node = Node::new(num_micro_batches)?;
    node.connect(get_node_addr()?)?;
    while !node.can_send() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Sender is ready");

    // Send messages
    for token_idx in 0..num_new_tokens {
        let mut handles = Vec::new();
        for micro_batch_idx in 0..num_micro_batches {
            // Send message
            println!("Sending token idx {} micro batch idx {}", token_idx, micro_batch_idx);
            let msg = vec![token_idx as u8; 1024 * 1024 * 50];
            let send_handle = node.isend(msg, micro_batch_idx);
            handles.push(send_handle);
        }
        for handle in handles {
            let _ = handle.wait();
        }
    }


    // Close the sender
    node.close()?;

    Ok(())
}

fn run_receiver() -> Result<()> {
    // Setup
    let num_new_tokens = 4;
    let num_micro_batches = 2;

    // Create receiver
    let mut node = Node::new(num_micro_batches)?;
    println!("cargo run sender {}", node.node_id);
    while !node.can_recv() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Receiver is ready");

    // Receive messages
    for token_idx in 0..num_new_tokens {
        let mut handles = Vec::new();
        for micro_batch_idx in 0..num_micro_batches {
            let recv_handle = node.irecv(micro_batch_idx);
            handles.push(recv_handle);
        }
        for (micro_batch_idx, handle) in handles.into_iter().enumerate() {
            let _msg = handle.wait()?;
            println!("Received token idx {} micro batch idx {}", token_idx, micro_batch_idx);
        }
    }

    // Close the receiver
    node.close()?;

    Ok(())
}

fn input(prompt: &str) -> String {
    use std::io::{self, Write};

    // Print the prompt
    print!("{}", prompt);
    // Ensure the prompt is displayed before reading input
    io::stdout().flush().expect("Failed to flush stdout");
    
    // Read user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    // Trim the trailing newline and return
    input.trim().to_string()
}

fn run_node(rank: &str) -> Result<()> {
    let num_micro_batches = 2;
    let mut node = Node::new(num_micro_batches)?;
    println!("Connect to {}", node.node_id);
    let peer_id = input("Please enter the peer ID: ");
    let bytes = hex::decode(peer_id)?;
    let peer_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
    let peer_addr = NodeAddr::new(peer_id);
    node.connect(peer_addr)?;
    while !node.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    println!("Connected");
    if rank == "0" {
        let msg = vec![0; 10];
        node.isend(msg, 0).wait()?;
    } else {
        let msg = node.irecv(0).wait()?;
        println!("Received message: {:?}", msg);
    }

    node.close()?;
    Ok(())
}

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).unwrap_or_else(|| {
        eprintln!("cargo run [sender|receiver]");
        std::process::exit(1);
    });

    match mode.as_str() {
        "node" => {
            let rank = args.get(2).unwrap_or_else(|| {
                eprintln!("cargo run node <rank>");
                std::process::exit(1);
            });
            run_node(rank).unwrap()
        }
        "sender" => run_sender().unwrap(),
        "receiver" => run_receiver().unwrap(),
        _ => {
            eprintln!("Invalid mode: {}", mode);
            std::process::exit(1);
        }
    }
} 
/*
 * This example demonstrates bidirectional communication between two nodes.
 * For simplicity, we initialize with known seeds, so that the nodes can
 * automatically connect to each other with known connection strings.
 *
 * Run the receiver:
 *
 * `cargo run --example bidirectional rank0`
 *
 * Run the sender:
 *
 * `cargo run --example bidirectional rank1`
 */
use anyhow::Result;
use anyhow::anyhow;
use prime_iroh::node::Node;
use std::env;

fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} [sender|receiver]", args[0]);
        return Err(anyhow!("Usage: {} [sender|receiver]", args[0]));
    }

    let num_streams = 1;
    let num_messages = 5;
    let mode = &args[1];
    let mut node: Node;
    let rank: u64;
    let peer_id: &str;

    match mode.as_str() {
        "rank0" => {
            // Initialize variables for rank 0 (define rank 1's connection string)
            println!("Running rank 0");
            rank = 0;
            peer_id = "ff87a0b0a3c7c0ce827e9cada5ff79e75a44a0633bfcb5b50f99307ddb26b337";
        }
        "rank1" => {
            // Initialize variables for rank 1 (define rank 0's connection string)
            println!("Running rank 1");
            rank = 1;
            peer_id = "ee1aa49a4459dfe813a3cf6eb882041230c7b2558469de81f87c9bf23bf10a03";
        }
        _ => {
            return Err(anyhow!("Invalid mode. Use 'rank0' or 'rank1'"));
        }
    }

    // Initialize node with rank seed
    node = Node::with_seed(num_streams, Some(rank))?;

    // Wait until connection is established
    println!("Waiting for connection...");
    node.connect(peer_id, 10, 100)?;
    while !node.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Connected to peer!");

    // Receive messages
    for i in 0..num_messages {
        // Send message
        let send_msg = format!("Hello from rank {}", rank);
        let bytes = send_msg.as_bytes().to_vec();
        let send_work = node.isend(bytes, 0, Some(1000));

        // Receive message
        let recv_work = node.irecv(0);
        let bytes = recv_work.wait()?;
        let recv_msg = String::from_utf8_lossy(&bytes);
        println!("Received message {}: {:?}", i + 1, recv_msg);

        // Wait for send work to complete
        send_work.wait()?;
    }

    // Clean up
    node.close().unwrap();

    Ok(())
}

/*
 * This example demonstrates a simple unidirectional communication between two nodes.
 * 
 * `cargo run --example unidirectional receiver` - then `cargo run --example unidirectional sender`
 */
use std::env;
use iroh_py::node::Node;
use anyhow::anyhow;
use anyhow::Result;

fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} [sender|receiver]", args[0]);
        return Err(anyhow!("Usage: {} [sender|receiver]", args[0]));
    }

    let mode = &args[1];
    match mode.as_str() {
        "receiver" => {
            println!("Running in receiver mode");
            println!("Waiting for sender to connect...");
            
            let mut node = Node::with_seed(1, Some(42))?;
            // Wait until we can receive
            while !node.can_recv() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            println!("Ready to receive!");

            // Receive 5 messages
            for i in 0..5 {
                let msg = node.irecv(0).wait()?;
                println!("Received message {}: {:?}", i, String::from_utf8_lossy(&msg));
            }
            // Clean up
            node.close().unwrap();
        }
        "sender" => {
            println!("Running in sender mode");
            let mut node = Node::with_seed(1, None)?;
            let receiver_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454";

            println!("Connecting to receiver...");
            node.connect(receiver_id)?;
            
            // Wait until we can send
            while !node.can_send() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            println!("Connected and ready to send!");

            // Send 5 messages
            for i in 0..5 {
                let msg = format!("Message {}", i);
                node.isend(msg.as_bytes().to_vec(), 0).wait()?;
                println!("Sending: {:?}", msg);
            }
            // Clean up
            node.close().unwrap();
        }
        _ => {
            println!("Invalid mode. Use 'sender' or 'receiver'");
        }
    }


    Ok(())
}

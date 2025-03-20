pub mod work;
mod sender;
mod receiver;

use anyhow::Result;
use iroh::{NodeId, NodeAddr};

use sender::IrohSender;
use receiver::Receiver;

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
    let mut sender = IrohSender::new()?;
    sender.connect(get_node_addr()?, num_micro_batches)?;
    println!("Sender is ready");

    // Send messages
    for token_idx in 0..num_new_tokens {
        let mut handles = Vec::new();
        for micro_batch_idx in 0..num_micro_batches {
            // Send message
            println!("Sending token idx {} micro batch idx {}", token_idx, micro_batch_idx);
            let msg = vec![token_idx as u8; 1024 * 1024 * 50];
            let send_handle = sender.isend(msg, micro_batch_idx);
            handles.push(send_handle);
        }
        for handle in handles {
            let _ = handle.wait();
        }
    }


    // Close the sender
    let _ = sender.close();

    Ok(())
}

fn run_receiver() -> Result<()> {
    // Setup
    let num_new_tokens = 4;
    let num_micro_batches = 2;

    // Create receiver
    let mut receiver = Receiver::new(num_micro_batches)?;
    while !receiver.is_ready() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Receiver is ready");

    // Receive messages
    for token_idx in 0..num_new_tokens {
        let mut handles = Vec::new();
        for micro_batch_idx in 0..num_micro_batches {
            let recv_handle = receiver.irecv(micro_batch_idx);
            handles.push(recv_handle);
        }
        for (micro_batch_idx, handle) in handles.into_iter().enumerate() {
            let _msg = handle.wait()?;
            println!("Received token idx {} micro batch idx {}", token_idx, micro_batch_idx);
        }
    }

    // Close the receiver
    receiver.close()?;

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
        "sender" => run_sender().unwrap(),
        "receiver" => run_receiver().unwrap(),
        _ => {
            eprintln!("Invalid mode: {}", mode);
            std::process::exit(1);
        }
    }
} 
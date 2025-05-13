use anyhow::Result;
use prime_iroh::node::Node;
use std::time::Duration;

const NUM_MESSAGES: usize = 5;
const NUM_STREAMS: usize = 1;

struct UnidirectionalTest {
    receiver: Node,
    sender: Node,
}

impl UnidirectionalTest {
    fn new() -> Result<Self> {
        // Initialize receiver
        let receiver = Node::new(NUM_STREAMS)?;
        println!("Initialized receiver (ID: {})", receiver.node_id());

        // Initialize sender
        let mut sender = Node::new(NUM_STREAMS)?;
        println!("Initialized sender (ID: {})", sender.node_id());

        // Wait for nodes to initialize (only necessary in single process tests)
        std::thread::sleep(Duration::from_millis(1000));

        // Connect sender to receiver
        println!(
            "Connecting sender->receiver (ID: {}->{})",
            sender.node_id(),
            receiver.node_id()
        );
        sender.connect(receiver.node_id(), 10)?;

        // Wait for connection to be established
        while !receiver.can_recv() || !sender.can_send() {
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(Self { receiver, sender })
    }

    fn test_sync_messages(&mut self) -> Result<()> {
        // Send messages synchronously
        for i in 0..NUM_MESSAGES {
            // Send message
            let msg = format!("Sync message {}", i);
            self.sender
                .isend(msg.as_bytes().to_vec(), 0, None)?
                .wait()?;
            println!("Sender sent: {}", msg);

            // Receive message
            let received = self.receiver.irecv(0)?.wait()?;
            let received_str = String::from_utf8_lossy(&received);
            println!("Receiver received: {}", received_str);

            // Verify received message matches sent message
            assert_eq!(received_str, msg);
        }

        Ok(())
    }

    fn test_async_messages(&mut self) -> Result<()> {
        // Send messages asynchronously
        for i in 0..NUM_MESSAGES {
            // Send message
            let msg = format!("Async message {}", i);
            let send_work = self.sender.isend(msg.as_bytes().to_vec(), 0, None);

            // Receive message
            let received = self.receiver.irecv(0)?.wait()?;
            let received_str = String::from_utf8_lossy(&received);

            // Verify received message matches sent message
            assert_eq!(received_str, msg);

            send_work?.wait()?;
        }

        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        self.sender.close()?;
        self.receiver.close()?;
        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_unidirectional_communication() -> Result<()> {
        let mut test = UnidirectionalTest::new()?;

        // Run sync message test
        test.test_sync_messages()?;

        // Run async message test
        test.test_async_messages()?;

        // Teardown
        test.teardown()?;

        Ok(())
    }
}

import pytest
from iroh_py import Node
import time

NUM_MESSAGES = 5
NUM_STREAMS = 1

class UnidirectionalTest:
    def __init__(self):
        # Initialize receiver
        self.receiver = Node(num_micro_batches=NUM_STREAMS)
        
        # Wait for receiver to be ready
        time.sleep(1)
        
        # Initialize sender
        self.sender = Node(num_micro_batches=NUM_STREAMS)
        self.sender.connect(self.receiver.node_id())
        
        # Wait for connection to be established
        while not self.receiver.can_recv() or not self.sender.can_send():
            time.sleep(0.1)

    def test_sync_messages(self):
        # Send messages synchronously
        for i in range(NUM_MESSAGES):
            # Send message
            msg = f"Sync message {i}"
            self.sender.isend(msg.encode(), 0, None).wait()
            print(f"Sender sent: {msg}")
            
            # Receive message
            received = self.receiver.irecv(0).wait()
            received_str = received.decode()
            print(f"Receiver received: {received_str}")
            
            # Verify received message matches sent message
            assert received_str == msg

    def test_async_messages(self):
        # Send messages asynchronously
        for i in range(NUM_MESSAGES):
            # Send message 
            msg = f"Async message {i}"
            send_work = self.sender.isend(msg.encode(), 0, None)

            # Receive message
            received = self.receiver.irecv(0).wait()
            received_str = received.decode()

            # Verify received message matches sent message
            assert received_str == msg

            send_work.wait()

def test_unidirectional_communication():
    test = UnidirectionalTest()
    
    # Test basic connection state
    assert test.receiver.can_recv()
    assert not test.receiver.can_send()
    assert test.sender.can_send()
    assert not test.sender.can_recv()
    
    # Run sync message test
    test.test_sync_messages()
    
    # Run async message test
    test.test_async_messages()

import pytest
from prime_iroh import Node
import time

NUM_MESSAGES = 5
NUM_STREAMS = 1

class UnidirectionalTest:
    def __init__(self):
        # Initialize receiver
        self.receiver = Node(num_streams=NUM_STREAMS)
        
        # Wait for receiver to be ready
        time.sleep(1)
        
        # Initialize sender
        self.sender = Node(num_streams=NUM_STREAMS)
        self.sender.connect(self.receiver.node_id(), 10, 1000)
        
        # Wait for connection to be established
        while not self.receiver.can_recv() or not self.sender.can_send():
            time.sleep(0.1)

    def test_sync_messages(self):
        # Send messages synchronously
        for i in range(NUM_MESSAGES):
            # Send message
            msg = f"Sync message {i}"
            self.sender.isend(msg.encode(), tag=0, latency=None).wait()
            
            # Receive message
            recv = self.receiver.irecv(tag=0).wait().decode()
            
            # Verify received message matches sent message
            assert recv == msg

    def test_async_messages(self):
        # Send messages asynchronously
        for i in range(NUM_MESSAGES):
            # Send async and receive sync
            msg = f"Async message {i}"
            sent = self.sender.isend(msg.encode(), tag=0, latency=None)
            recv = self.receiver.irecv(tag=0).wait().decode()

            # Verify received message matches sent message
            assert msg == recv

            sent.wait()

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

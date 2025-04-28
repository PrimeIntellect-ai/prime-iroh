from prime_iroh import Node
import time

NUM_MESSAGES = 5
NUM_STREAMS = 1

class BidirectionalTest:
    def __init__(self):
        # Initialize nodes
        self.node0 = Node.with_seed(NUM_STREAMS, None)
        self.node1 = Node.with_seed(NUM_STREAMS, None)
        
        # Wait for nodes to be ready
        time.sleep(1)

        # Connect bidirectionally
        self.node0.connect(self.node1.node_id())
        self.node1.connect(self.node0.node_id())
        
        # Wait for connection to be established
        while not self.node0.can_recv() or not self.node1.can_send():
            time.sleep(0.1)

    def test_communication(self):
        for i in range(NUM_MESSAGES):
            self.node0.isend(f"Message {i} from node 0".encode(), 0, None)
            self.node1.isend(f"Message {i} from node 1".encode(), 0, None)

            received_from_node0 = self.node1.irecv(0).wait()
            received_from_node1 = self.node0.irecv(0).wait()

            assert received_from_node0 == f"Message {i} from node 0".encode()
            assert received_from_node1 == f"Message {i} from node 1".encode()

def test_bidirectional_communication():
    test = BidirectionalTest()
    
    # Test basic connection state
    assert test.node0.is_ready()
    assert test.node1.is_ready()
    assert test.node0.can_send()
    assert test.node1.can_send()
    assert test.node0.can_recv()
    assert test.node1.can_recv()

    # Test bidirectional communication
    test.test_communication()

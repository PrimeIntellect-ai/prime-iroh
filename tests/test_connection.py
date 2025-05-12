import pytest
from prime_iroh import Node
import time

NUM_STREAMS = 1

class ConnectionTest:
    def __init__(self, num_nodes):
        # Initialize nodes
        self.nodes = []
        
        for i in range(num_nodes):
            node = Node(num_streams=NUM_STREAMS)
            node_id = node.node_id()
            print(f"Initializing node {i} (ID: {node_id})")
            self.nodes.append(node)
        
        # Connect nodes
        for i in range(num_nodes):
            current_node = self.nodes[i]
            j = (i + 1) % num_nodes
            node_id = current_node.node_id()
            peer_id = self.nodes[j].node_id()
            print(f"Connecting node {i}->{j} (ID: {node_id}->{peer_id})")
            current_node.connect(peer_id, 10, 100)
        
        # Wait for all nodes to be ready
        while not all(node.is_ready() for node in self.nodes):
            time.sleep(0.1)

    def verify_active_connection_state(self):
        for node in self.nodes:
            assert node.is_ready(), "Node should be ready"
            assert node.can_send(), "Node should be able to send"
            assert node.can_recv(), "Node should be able to receive"
        print("All nodes are active")

    def verify_inactive_connection_state(self):
        for node in self.nodes:
            assert not node.is_ready(), "Node should not be ready"
            assert not node.can_send(), "Node should not be able to send"
            assert not node.can_recv(), "Node should not be able to receive"
        print("All nodes are inactive")

    def teardown(self):
        for i, node in enumerate(self.nodes):
            print(f"Closing node {i}")
            node.close()

def test_connection():
    num_nodes = 2
    test = ConnectionTest(num_nodes)
    
    # Test connection state
    test.verify_active_connection_state()
    
    # Teardown
    test.teardown()
    
    # Test connection state
    test.verify_inactive_connection_state() 
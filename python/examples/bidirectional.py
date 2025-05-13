#!/usr/bin/env python3
"""
This example demonstrates bidirectional communication between two nodes.
For simplicity, we initialize with known seeds, so that the nodes can
automatically connect to each other with known connection strings.

Run the receiver:
    python bidirectional.py rank0

Run the sender:
    python bidirectional.py rank1
"""

import sys
import time
import logging
from prime_iroh import Node

def main():
    # Initialize logging
    logging.basicConfig(level=logging.INFO)
    
    # Get command line arguments
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} [rank0|rank1]")
        sys.exit(1)
        
    num_streams = 1
    num_messages = 5
    mode = sys.argv[1]
    
    if mode == "rank0":
        # Initialize variables for rank 0 (define rank 1's connection string)
        print("Running rank 0")
        rank = 0
        peer_id = "ff87a0b0a3c7c0ce827e9cada5ff79e75a44a0633bfcb5b50f99307ddb26b337"
    elif mode == "rank1":
        # Initialize variables for rank 1 (define rank 0's connection string)
        print("Running rank 1")
        rank = 1
        peer_id = "ee1aa49a4459dfe813a3cf6eb882041230c7b2558469de81f87c9bf23bf10a03"
    else:
        print("Invalid mode. Use 'rank0' or 'rank1'")
        sys.exit(1)
        
    # Initialize node with rank seed
    node = Node.with_seed(num_streams, seed=rank)
    
    # Wait until connection is established
    print("Waiting for connection...")
    node.connect(peer_id, 10, 100)
    while not node.is_ready():
        time.sleep(0.1)
    print("Connected to peer!")
    
    # Send and receive messages
    for i in range(num_messages):
        # Send message
        send_msg = f"Hello from rank {rank}"
        bytes_data = send_msg.encode('utf-8')
        send_work = node.isend(bytes_data, 0, 1000)
        
        # Receive message
        recv_work = node.irecv(0)
        bytes_data = recv_work.wait()
        recv_msg = bytes_data.decode('utf-8')
        print(f"Received message {i + 1}: {recv_msg}")
        
        # Wait for send work to complete
        send_work.wait()
    
    # Clean up
    node.close()

if __name__ == "__main__":
    main() 
#!/usr/bin/env python3
"""
This example demonstrates simple unidirectional communication between two nodes - one sender and one receiver.

Run the receiver:
    python unidirectional.py receiver

Run the sender:
    python unidirectional.py sender
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
        print(f"Usage: {sys.argv[0]} [sender|receiver]")
        sys.exit(1)
        
    num_streams = 1
    num_messages = 5
    mode = sys.argv[1]
    
    if mode == "receiver":
        # Run the receiver
        print("Running receiver")
        node = Node.with_seed(num_streams, seed=42)
        
        # Wait for incoming connection
        print("Waiting for receiver to be ready...")
        while not node.can_recv():
            time.sleep(0.1)
        print("Ready to receive!")
        
        # Receive messages
        for i in range(num_messages):
            bytes_data = node.irecv(0).wait()
            msg = bytes_data.decode('utf-8')
            print(f"Received message {i + 1}: {msg}")
            
    elif mode == "sender":
        # Run the sender
        print("Running sender")
        node = Node.with_seed(num_streams, seed=None)
        
        # Connect to receiver
        print("Connecting to receiver...")
        receiver_id = "9bdb607f02802cdd126290cfa1e025e4c13bbdbb347a70edeace584159303454"
        node.connect(receiver_id, 10, 100)
        
        # Wait for connection to be established
        print("Waiting for sender to be ready...")
        while not node.can_send():
            time.sleep(0.1)
        print("Ready to send!")
        
        # Send messages
        for i in range(num_messages):
            msg = "Hello from sender"
            bytes_data = msg.encode('utf-8')
            node.isend(bytes_data, 0, 1000).wait()  # 1s artificial latency
            print(f"Sent message {i + 1}: {msg}")
            
    else:
        print(f"Invalid mode. Use 'sender' or 'receiver'")
        sys.exit(1)
        
    # Clean up
    print("Closing node...")
    node.close()

if __name__ == "__main__":
    main() 
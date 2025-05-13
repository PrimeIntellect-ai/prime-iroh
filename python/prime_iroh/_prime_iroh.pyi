from typing import Optional

class SendWork:
    """A class representing the future of an asynchronous send operation."""
    def wait(self) -> None:
        """Wait for the send operation to complete.
        
        Returns:
            None
        
        Raises:
            RuntimeError: If the operation fails
        """
        ...

class RecvWork:
    """A class representing the future of an asynchronous receive operation."""
    def wait(self) -> bytes:
        """Wait for the receive operation to complete and return the received data.
        
        Returns:
            bytes: The received data
            
        Raises:
            RuntimeError: If the operation fails
        """
        ...

class Node:
    """A class combining a single-peer sender/receiver, allowing to send to exactly one 
    and receive from exactly one (potentially different) peer."""
    
    def __init__(self, num_streams: int) -> None:
        """Create a new Node with a given number of micro-batches.
        
        Args:
            num_streams: The number of parallel streams to use
            
        Raises:
            RuntimeError: If node creation fails
        """
        ...
    
    @staticmethod
    def with_seed(num_streams: int, seed: Optional[int] = None) -> "Node":
        """Create a new Node with a given number of micro-batches and fixed seed.
        
        Args:
            num_streams: The number of parallel streams to use
            seed: Optional seed for generating the secret/public key
            
        Returns:
            Node: A new Node instance
            
        Raises:
            RuntimeError: If node creation fails
        """
        ...
    
    def node_id(self) -> str:
        """Get the node ID of the Node.
        
        Returns:
            str: The node ID as a hex string
        """
        ...
    
    def connect(self, peer_id_str: str, num_retries: int) -> None:
        """Connect to a Node with a given node ID.
        
        Args:
            peer_id_str: The ID of the peer to connect to
            num_retries: The number of retries to attempt
            
        Raises:
            RuntimeError: If connection fails
        """
        ...
    
    def can_recv(self) -> bool:
        """Check if the Node can receive messages.
        
        Returns:
            bool: True if the Node can receive messages
        """
        ...
    
    def can_send(self) -> bool:
        """Check if the Node can send messages.
        
        Returns:
            bool: True if the Node can send messages
        """
        ...
    
    def is_ready(self) -> bool:
        """Check if the Node is ready to send and receive messages.
        
        Returns:
            bool: True if the Node is ready for both sending and receiving
        """
        ...
    
    def isend(self, msg: bytes, tag: int, latency: Optional[int] = None) -> SendWork:
        """Send a message to a Node with a given tag.
        
        Args:
            msg: The message to send as bytes
            tag: The tag to send the message to
            latency: Optional latency in milliseconds
            
        Returns:
            SendWork: A SendWork object representing the async operation
            
        Raises:
            RuntimeError: If sending fails
        """
        ...
    
    def irecv(self, tag: int) -> RecvWork:
        """Receive a message from a Node with a given tag.
        
        Args:
            tag: The tag to receive the message from
            
        Returns:
            RecvWork: A RecvWork object representing the async operation
            
        Raises:
            RuntimeError: If receiving fails
        """
        ...
    
    def close(self) -> None:
        """Close the Node.
        
        Raises:
            RuntimeError: If closing fails
        """
        ...
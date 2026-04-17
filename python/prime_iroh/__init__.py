from ._prime_iroh import Node, SendWork, RecvWork

# Copy docstrings from the .pyi stub file so help() works at runtime.
# The compiled Rust extension (PyO3) does not embed these docstrings.

SendWork.__doc__ = """A class representing the future of an asynchronous send operation."""

SendWork.wait.__doc__ = """Wait for the send operation to complete.

Returns:
    None

Raises:
    RuntimeError: If the operation fails
"""

RecvWork.__doc__ = """A class representing the future of an asynchronous receive operation."""

RecvWork.wait.__doc__ = """Wait for the receive operation to complete and return the received data.

Returns:
    bytes: The received data

Raises:
    RuntimeError: If the operation fails
"""

Node.__doc__ = """A class combining a single-peer sender/receiver, allowing to send to exactly one
and receive from exactly one (potentially different) peer."""

Node.__init__.__doc__ = """Create a new Node with a given number of micro-batches.

Args:
    num_streams: The number of parallel streams to use

Raises:
    RuntimeError: If node creation fails
"""

Node.with_seed.__func__.__doc__ = """Create a new Node with a given number of micro-batches and fixed seed.

Args:
    num_streams: The number of parallel streams to use
    seed: Optional seed for generating the secret/public key

Returns:
    Node: A new Node instance

Raises:
    RuntimeError: If node creation fails
"""

Node.node_id.__doc__ = """Get the node ID of the Node.

Returns:
    str: The node ID as a hex string
"""

Node.connect.__doc__ = """Connect to a Node with a given node ID.

Args:
    peer_id_str: The ID of the peer to connect to
    num_retries: The number of retries to attempt

Raises:
    RuntimeError: If connection fails
"""

Node.can_recv.__doc__ = """Check if the Node can receive messages.

Returns:
    bool: True if the Node can receive messages
"""

Node.can_send.__doc__ = """Check if the Node can send messages.

Returns:
    bool: True if the Node can send messages
"""

Node.is_ready.__doc__ = """Check if the Node is ready to send and receive messages.

Returns:
    bool: True if the Node is ready for both sending and receiving
"""

Node.isend.__doc__ = """Send a message to a Node with a given tag.

Args:
    msg: The message to send as bytes
    tag: The tag to send the message to
    latency: Optional latency in milliseconds

Returns:
    SendWork: A SendWork object representing the async operation

Raises:
    RuntimeError: If sending fails
"""

Node.irecv.__doc__ = """Receive a message from a Node with a given tag.

Args:
    tag: The tag to receive the message from

Returns:
    RecvWork: A RecvWork object representing the async operation

Raises:
    RuntimeError: If receiving fails
"""

Node.close.__doc__ = """Close the Node.

Raises:
    RuntimeError: If closing fails
"""

__all__ = ["Node", "SendWork", "RecvWork"]

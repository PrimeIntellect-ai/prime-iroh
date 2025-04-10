<p align="center">
</p>

<img src="https://github.com/user-attachments/assets/51e44795-5206-49d6-a12a-ecacd2799df2" alt="Prime Intellect" style="width: 100%; height: auto;"/>

---

<p align="center">

<h3 align="center">
P2P Pipeline Parallel Communication
</h3>

---

This codebase exposes a Python interface for reliable, asynchronous peer-to-peer communication built upon [Iroh](https://github.com/iroh-project/iroh). The core classes exposed are:

- `Node`: A class combining a single-peer sender/ receiver in one class, allowing to send to exactly *one* and receive from exactly *one* (potentially different) peer. The class allows for concurrent communication by opening multiple, consistent streams.
- `Work`: A class representing the future of an asynchronous operation, that can be awaited using a `wait` method.

Because we are building on top of Iroh, we get many nice networking features out of the box. Most importantly, the library guarantees reliable P2P connections between nodes, trying to establish directions connections whenever possible, and falling back to NAT-hole punching and relaying when necessary. The API is mirroring the way asynchronous communication is handled in `torch.distributed`, i.e. exposing `isend` and `irecv` that return work objects that can be awaited using a `wait` method. This allows for a clean integration with the rest of the PyTorch ecosystem. For an example of this check out our research codebase for [pipeline parallel inference](https://github.com/primeintellect-ai/pipelined-gpt-fast) that uses this library for P2P communication across geographically distributed nodes.


## Installation

**Quick Install**: Run the following command for a quick install:

```bash
curl -sSL https://raw.githubusercontent.com/PrimeIntellect-ai/iroh_py/refs/heads/main/script/install.sh | bash
```

**Manual Install**: First, install uv and cargo to build the project.

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
source $HOME/.local/bin/env
```

```bash
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```

Then, clone the repository

```bash
git clone git@github.com:PrimeIntellect-ai/iroh_py.git && cd iroh_py
```

To build the Rust backend run `cargo build`, to build the Python bindings run `uv sync`. This will let you install `iroh-py` as a Python package within the virtual environment. To use the library in your project, you can import it as follows:

## Usage

You can find the basic usage examples in the `examples` directory showing unidirectional and bidirectional communication patterns in Rust. 

Run unidirectional communication example:

```bash
cargo run --example unidirectional
```

Run bidirectional communication example:

```bash
cargo run --example bidirectional
```

For Python usage, you would use the node class as follows:

```python
# On the receiver side
from iroh_py import Node

# Initialize the node
node = Node(num_streams=1)
print(f"Connect to: {node.node_id()}")

# Wait for sender to connect
while not node.can_recv():
    time.sleep(0.1)

# Receive message
msg = node.irecv(tag=0).wait()
```

```python
# On the sender side
from iroh_py import Node

# Initialize the node
node = Node(num_streams=1)

# Connect to the receiver
node.connect(peer_id=...)

# Wait for connection to be established
while not node.can_send():
    time.sleep(0.1)

# Send message
node.isend("Hello, world!".encode(), tag=0, latency=None).wait()
```

## Tests

We include unit tests and integration tests for the Rust backend which can be run using `cargo test`. The integration tests for uni- and bidirectional communication are also ported to Python and can be run using `uv run pytest`.
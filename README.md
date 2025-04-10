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

- `Sender`: A class that allows you to send messages to a remote peer.
- `Receiver`: A class that allows you to receive messages from a remote peer.
- `Work`: A class representing the future of an asynchronous operation, that can be awaited using a `wait` method.
- `Node`: A sender/ receiver in one class, that allows to send to exactly *one* and receive from exactly *one* (potentially different) peer. The class allows for concurrent communication by opening multiple, consistent streams.

Because we are building on top of Iroh, we get many nice networking features out of the box. Most importantly, the library guarantees reliable P2P connections between nodes, trying to establish directions connections whenever possible, and falling back to NAT-hole punching and relaying when necessary. The API is designed to around the way asynchronous communication is handled in `torch.distributed`, i.e. exposing `isend` and `irecv` that return work objects that can be awaited using a `wait` method. This allows for a clean integration with the rest of the PyTorch ecosystem. For an example of this check out our research codebase for [pipeline parallel inference](https://github.com/primeintellect-ai/pipelined-gpt-fast) that uses this library for P2P communication across geographically distributed nodes.


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

You can find the basic usage examples in the `examples` directory in Rust. For example, to run a simple uni-directional send/ receive operation, you can use the following code:

```bash
cargo run --example unidirectional
```

## Tests

We include simple tests for both the Rust backend and the Python bindings.

To test the Rust backend run

```bash
cargo test
```

To test the Python bindings run

```bash
uv run pytest
```


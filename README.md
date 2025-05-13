<p align="center">
</p>

<img src="https://github.com/user-attachments/assets/51e44795-5206-49d6-a12a-ecacd2799df2" alt="Prime Intellect" style="width: 100%; height: auto;"/>

---

<p align="center">

<h3 align="center">
PRIME-IROH: P2P Pipeline Parallel Communication
</h3>

---

This library exposes a Python interface for reliable, asynchronous peer-to-peer communication built upon [Iroh](https://github.com/iroh-project/iroh). The core classes exposed are:

- `Node`: A class combining a single-peer sender/ receiver in one class, allowing to send to exactly *one* and receive from exactly *one* (potentially different) peer. The class allows for concurrent communication by opening multiple, consistent streams.
- `Work`: A class representing the future of an asynchronous operation, that can be awaited using a `wait` method.

Because we are building on top of Iroh, we get many nice networking features out of the box. Most importantly, the library guarantees reliable P2P connections between nodes, trying to establish directions connections whenever possible, and falling back to NAT-hole punching and relaying when necessary. The API is mirroring the way asynchronous communication is handled in `torch.distributed`, i.e. exposing `isend` and `irecv` that return work objects that can be awaited using a `wait` method. This allows for a clean integration with the rest of the PyTorch ecosystem.


## Installation

**Quick Install**: Run the following command for a quick install:

```bash
curl -sSL https://raw.githubusercontent.com/PrimeIntellect-ai/prime-iroh/refs/heads/master/script/install.sh | bash
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
git clone git@github.com:PrimeIntellect-ai/prime-iroh.git && cd prime-iroh
```

To build the Rust backend run `cargo build`, to build the Python bindings run `uv sync`. This will let you install `prime-iroh` as a Python package within the virtual environment.

## Examples

You can find the basic usage examples in the `examples` directory showing unidirectional and bidirectional communication patterns in Rust and Python.

Run unidirectional communication example:

```bash
# Rust
cargo run --example unidirectional
```

```bash
# Python
uv run python examples/unidirectional.py
```

Run bidirectional communication example:

```bash
# Rust
cargo run --example bidirectional
```

```bash
# Python
uv run python examples/bidirectional.py
```

*You can set the log level by setting the `RUST_LOG` environment variable. For example, to see info logs from the `prime-iroh` crate, set `RUST_LOG=prime_iroh=info`.*

For Python usage, you would use the node class as follows:

## Tests

We include unit tests and integration tests for the Rust backend which can be run using `cargo test`. The integration tests for uni- and bidirectional communication are also ported to Python and can be run using `uv run pytest`.

**Rust Tests**

Run full test suite (unit and integration tests):

```bash
cargo test
```

*Note: To run the tests with verbose output, use `cargo test -- --nocapture`.*

Run single unit test, e.g. tests in `src/node.rs` (this will pattern match the test function names):

```bash
cargo test node
```

Run single integration test, e.g. `tests/connection.rs`:

```bash
cargo test --test connection
```

**Python Tests**

Run full test suite (only integration tests are available for now):

```bash
uv run pytest
```

To run the tests with verbose output, use `uv run pytest -s`.

Run single test, e.g. `pytests/test_unidirectional.py`:

```bash
uv run pytest tests/test_unidirectional.py
```

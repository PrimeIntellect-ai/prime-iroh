/*!
 * Iroh P2P library
 *
 * A library for P2P networking functionality using Iroh.
 */

pub mod bindings;
pub mod node;
pub mod receiver;
pub mod sender;
pub mod work;
use std::sync::Once;

// Rust library
pub use node::Node;
pub use receiver::Receiver;
pub use sender::Sender;
pub use work::{RecvWork, SendWork};

// Python bindings
use pyo3::prelude::*;

static INIT: Once = Once::new();

// Expose classes to Python
#[pymodule]
fn prime_iroh(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize logging via environment variables
    INIT.call_once(|| {
        env_logger::init();
    });

    m.add_class::<bindings::SendWork>()?;
    m.add_class::<bindings::RecvWork>()?;
    m.add_class::<bindings::Node>()?;
    Ok(())
}

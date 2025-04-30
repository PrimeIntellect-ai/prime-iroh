/*!
 * Iroh P2P library
 * 
 * A library for P2P networking functionality using Iroh.
 */

pub mod work;
pub mod sender;
pub mod receiver;
pub mod node;
pub mod bindings;
 
// Rust library
pub use work::{SendWork, RecvWork};
pub use sender::Sender;
pub use receiver::Receiver;
pub use node::Node;

// Python bindings
use pyo3::prelude::*;

// Expose classes to Python
#[pymodule]
fn prime_iroh(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<bindings::SendWork>()?;
    m.add_class::<bindings::RecvWork>()?;
    m.add_class::<bindings::Node>()?;
    Ok(())
}

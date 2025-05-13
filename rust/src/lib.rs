/*!
 * PRIME-IROH
 *
 * Asynchronous P2P communication backend for decentralized pipeline parallelism,
 * built on top of Iroh.
 */

// Modules
pub mod node;
pub mod receiver;
pub mod sender;
pub mod work;
use crate::node::Node as IrohNode;
use crate::work::{RecvWork as IrohRecvWork, SendWork as IrohSendWork};

// Miscellaneous
use anyhow::Result;
use std::sync::RwLock;

// Bindings
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass]
pub struct SendWork {
    inner: RwLock<Option<Result<IrohSendWork>>>,
}

impl SendWork {
    pub fn new(inner: Result<IrohSendWork>) -> Self {
        Self {
            inner: RwLock::new(Some(inner)),
        }
    }
}

#[pymethods]
impl SendWork {
    /// Wait for the work to complete and return the result
    pub fn wait(&self) -> PyResult<()> {
        // Take the inner value out of the RwLock, leaving None in its place
        let mut write_guard = self
            .inner
            .write()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        if let Some(inner) = write_guard.take() {
            inner
                .unwrap()
                .wait()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Err(PyRuntimeError::new_err(
                "SendWork has already been consumed",
            ))
        }
    }
}

#[pyclass]
pub struct RecvWork {
    inner: RwLock<Option<Result<IrohRecvWork>>>,
}

// Completely outside the pymethods - not exposed to Python
impl RecvWork {
    pub fn new(inner: Result<IrohRecvWork>) -> Self {
        Self {
            inner: RwLock::new(Some(inner)),
        }
    }
}

#[pymethods]
impl RecvWork {
    pub fn wait(&self) -> PyResult<Vec<u8>> {
        // Take the inner value out of the RwLock, leaving None in its place
        let mut write_guard = self
            .inner
            .write()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        if let Some(inner) = write_guard.take() {
            inner
                .unwrap()
                .wait()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Err(PyRuntimeError::new_err(
                "RecvWork has already been consumed",
            ))
        }
    }
}

#[pyclass]
pub struct Node {
    inner: IrohNode,
}

#[pymethods]
impl Node {
    #[new]
    pub fn new(num_streams: usize) -> PyResult<Self> {
        Ok(Self {
            inner: IrohNode::new(num_streams)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        })
    }

    #[staticmethod]
    pub fn with_seed(num_streams: usize, seed: Option<u64>) -> PyResult<Self> {
        Ok(Self {
            inner: IrohNode::with_seed(num_streams, seed)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        })
    }

    pub fn node_id(&self) -> String {
        self.inner.node_id().to_string()
    }

    pub fn connect(&mut self, peer_id_str: String, num_retries: usize) -> PyResult<()> {
        self.inner
            .connect(peer_id_str, num_retries)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    pub fn can_recv(&self) -> bool {
        self.inner.can_recv()
    }

    #[pyo3(text_signature = "()")]
    pub fn can_send(&self) -> bool {
        self.inner.can_send()
    }

    #[pyo3(text_signature = "()")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    pub fn isend(
        &mut self,
        msg: Vec<u8>,
        tag: usize,
        latency: Option<usize>,
    ) -> PyResult<SendWork> {
        Ok(SendWork::new(self.inner.isend(msg, tag, latency)))
    }

    pub fn irecv(&mut self, tag: usize) -> PyResult<RecvWork> {
        Ok(RecvWork::new(self.inner.irecv(tag)))
    }

    pub fn close(&mut self) -> PyResult<()> {
        self.inner
            .close()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

// Initialize logging via environment variables
use std::sync::Once;
static INIT: Once = Once::new();

// Expose classes to Python
#[pymodule]
fn _prime_iroh(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize logging via environment variables
    INIT.call_once(|| {
        env_logger::init();
    });

    m.add_class::<SendWork>()?;
    m.add_class::<RecvWork>()?;
    m.add_class::<Node>()?;
    Ok(())
}

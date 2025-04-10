use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::sync::RwLock;
use crate::node::Node as IrohNode;
use crate::work::{SendWork as IrohSendWork, RecvWork as IrohRecvWork};

/// Formats the sum of two numbers as string.
#[pyfunction]
pub fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python wrapper around the Work class
#[pyclass]
pub struct SendWork {
    inner: RwLock<Option<IrohSendWork>>,
}

// Completely outside the pymethods - not exposed to Python
impl SendWork {
    pub fn new(inner: IrohSendWork) -> Self {
        Self { inner: RwLock::new(Some(inner)) }
    }
}

#[pymethods]
impl SendWork {
    /// Wait for the work to complete and return the result
    pub fn wait(&self) -> PyResult<()> {
        // Take the inner value out of the RwLock, leaving None in its place
        let mut write_guard = self.inner.write().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        if let Some(inner) = write_guard.take() {
            inner.wait().map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Err(PyRuntimeError::new_err("SendWork has already been consumed"))
        }
    }
}

/// A Python wrapper around the RecvWork class
#[pyclass]
pub struct RecvWork {
    inner: RwLock<Option<IrohRecvWork>>,
}

// Completely outside the pymethods - not exposed to Python
impl RecvWork {
    pub fn new(inner: IrohRecvWork) -> Self {
        Self { inner: RwLock::new(Some(inner)) }
    }
}

#[pymethods]
impl RecvWork {
    /// Wait for the work to complete and return the result
    pub fn wait(&self) -> PyResult<Vec<u8>> {
        // Take the inner value out of the RwLock, leaving None in its place
        let mut write_guard = self.inner.write().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        if let Some(inner) = write_guard.take() {
            inner.wait().map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Err(PyRuntimeError::new_err("RecvWork has already been consumed"))
        }
    }
}

/// A Python wrapper around the Node class
#[pyclass]
pub struct Node {
    inner: IrohNode,
}

#[pymethods]
impl Node {
    /// Create a new Node with a given number of micro-batches.
    ///
    /// Args:
    ///     num_streams: The number of parallel streams to use
    ///
    /// Returns:
    ///     A new Node object
    #[new]
    #[pyo3(text_signature = "(num_streams)")]
    pub fn new(num_streams: usize) -> PyResult<Self> {
        Ok(Self {
            inner: IrohNode::new(num_streams).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        })
    }

    /// Create a new Node with a given number of micro-batches and
    /// fixed seed for generating the secret/ public key. Useful for
    /// debugging purposes.
    ///
    /// Args:
    ///     num_streams: The number of parallel streams to use
    ///     seed: The seed to use for the Node
    ///
    /// Returns:
    ///     A new Node object
    #[staticmethod]
    #[pyo3(text_signature = "(num_streams, seed)")]
    pub fn with_seed(num_streams: usize, seed: Option<u64>) -> PyResult<Self> {
        Ok(Self {
            inner: IrohNode::with_seed(num_streams, seed).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        })
    }

    /// Get the node ID of the Node.
    ///
    /// Returns:
    ///     The node ID of the Node
    #[pyo3(text_signature = "()")]
    pub fn node_id(&self) -> String {
        self.inner.node_id().to_string()
    }

    /// Connect to a Node with a given node ID.
    ///
    /// Args:
    ///     node_id_str: The node ID of the Node to connect to
    ///
    /// Returns:
    ///     None if successful
    #[pyo3(text_signature = "(node_id_str)")]
    pub fn connect(&mut self, node_id_str: &str) -> PyResult<()> {
        self.inner.connect(node_id_str).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Check if the Node can receive messages.
    ///
    /// Returns:
    ///     True if the Node can receive messages, False otherwise
    #[pyo3(text_signature = "()")]
    pub fn can_recv(&self) -> bool {
        self.inner.can_recv()
    }

    /// Check if the Node can send messages.
    ///
    /// Returns:
    ///     True if the Node can send messages, False otherwise
    #[pyo3(text_signature = "()")]
    pub fn can_send(&self) -> bool {
        self.inner.can_send()
    }

    /// Check if the Node is ready to send and receive messages.
    ///
    /// Returns:
    ///     True if the Node is ready to send and receive messages, False otherwise
    #[pyo3(text_signature = "()")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Send a message to a Node with a given tag.
    ///
    /// Args:
    ///     msg: The message to send
    ///     tag: The tag to send the message to
    ///     latency: Optional latency in milliseconds (default: 0)
    ///
    /// Returns:
    ///     A SendWork object
    #[pyo3(text_signature = "(msg, tag, latency=None)")]
    pub fn isend(&mut self, msg: Vec<u8>, tag: usize, latency: Option<usize>) -> PyResult<SendWork> {
        Ok(SendWork::new(self.inner.isend(msg, tag, latency)))
    }

    /// Receive a message from a Node with a given tag.
    ///
    /// Args:
    ///     tag: The tag to receive the message from
    ///
    /// Returns:
    ///     A RecvWork object
    #[pyo3(text_signature = "(tag)")]
    pub fn irecv(&mut self, tag: usize) -> PyResult<RecvWork> {
        Ok(RecvWork::new(self.inner.irecv(tag)))
    }

    /// Close the Node.
    #[pyo3(text_signature = "()")]
    pub fn close(&mut self) -> PyResult<()> {
        self.inner.close().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}
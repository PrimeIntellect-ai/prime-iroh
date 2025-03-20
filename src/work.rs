use anyhow::Result;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

pub struct Work<T> {
    pub runtime: Arc<Runtime>,
    pub handle: JoinHandle<Result<T>>,
}

impl<T> Work<T> {
    pub fn wait(self) -> Result<T> {
        self.runtime.block_on(self.handle)?
    }
}
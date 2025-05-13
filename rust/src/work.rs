use anyhow::Result;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

pub struct SendWork {
    pub runtime: Arc<Runtime>,
    pub handle: JoinHandle<Result<()>>,
}

impl SendWork {
    pub fn new(runtime: Arc<Runtime>, handle: JoinHandle<Result<()>>) -> Self {
        Self { runtime, handle }
    }

    pub fn wait(self) -> Result<()> {
        self.runtime.block_on(self.handle)?
    }
}

pub struct RecvWork {
    pub runtime: Arc<Runtime>,
    pub handle: JoinHandle<Result<Vec<u8>>>,
}

impl RecvWork {
    pub fn new(runtime: Arc<Runtime>, handle: JoinHandle<Result<Vec<u8>>>) -> Self {
        Self { runtime, handle }
    }

    pub fn wait(self) -> Result<Vec<u8>> {
        self.runtime.block_on(self.handle)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use anyhow::Error;
    use tokio::time::sleep;

    #[test]
    fn test_work_success() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let handle = runtime.spawn(async {
            Ok(b"test".to_vec())
        });
        
        let work = RecvWork {
            runtime,
            handle,
        };
        
        let result = work.wait();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"test".to_vec());
    }

    #[test]
    fn test_work_error() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let handle = runtime.spawn(async {
            Err(Error::msg("test error"))
        });
        
        let work = RecvWork {
            runtime,
            handle,
        };
        
        let result = work.wait();
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "test error");
    }

    #[test]
    fn test_work_with_delay() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let handle = runtime.spawn(async {
            sleep(Duration::from_millis(100)).await;
            Ok(b"test".to_vec())
        });

        let work = RecvWork {
            runtime,
            handle,
        };
        
        let start = Instant::now();
        let result = work.wait();
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"test".to_vec());
        assert!(duration >= Duration::from_millis(100));
    }
}


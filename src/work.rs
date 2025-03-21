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
            Ok(42)
        });
        
        let work = Work {
            runtime,
            handle,
        };
        
        let result = work.wait();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_work_error() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let handle = runtime.spawn(async {
            Err(Error::msg("test error"))
        });
        
        let work: Work<()> = Work {
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
            Ok(100)
        });

        let work = Work {
            runtime,
            handle,
        };
        
        let start = Instant::now();
        let result = work.wait();
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert!(duration >= Duration::from_millis(100));
    }
}


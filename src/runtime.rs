use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Global runtime wrapper instance
static RUNTIME: OnceLock<EtcdRt> = OnceLock::new();

/// Counter for active tasks (for graceful shutdown)
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

/// Runtime wrapper that provides task tracking and graceful shutdown
///
/// This struct tracks active async tasks and waits for them to complete
/// during Python shutdown, preventing GIL state violations and segfaults.
///
/// Note: Tasks still run on pyo3_async_runtimes' runtime. This wrapper
/// only provides task tracking and a grace period for completion.
pub struct EtcdRt {
    // Marker struct - cleanup happens via Drop
}

impl EtcdRt {
    /// Create the global runtime wrapper
    fn new() -> Self {
        eprintln!("[etcd-client-py] Initializing task tracker...");
        EtcdRt {}
    }

    /// Get or initialize the global runtime wrapper
    pub fn get_or_init() -> &'static EtcdRt {
        RUNTIME.get_or_init(EtcdRt::new)
    }

    /// Spawn a future with task tracking
    ///
    /// This wraps the future to track when it starts and completes,
    /// enabling graceful shutdown by waiting for active tasks.
    pub fn spawn<'a, F, T>(&self, py: Python<'a>, fut: F) -> PyResult<Bound<'a, PyAny>>
    where
        F: std::future::Future<Output = PyResult<T>> + Send + 'static,
        T: for<'py> pyo3::IntoPyObject<'py> + Send + 'static,
    {
        // Increment active task counter
        ACTIVE_TASKS.fetch_add(1, Ordering::SeqCst);

        // Wrap the future to decrement counter on completion
        let wrapped_fut = async move {
            let result = fut.await;
            ACTIVE_TASKS.fetch_sub(1, Ordering::SeqCst);
            result
        };

        // Use pyo3_async_runtimes for Python integration
        pyo3_async_runtimes::tokio::future_into_py(py, wrapped_fut)
    }

    /// Wait for all active tasks to complete (with timeout)
    fn wait_for_tasks(&self, timeout_ms: u64) {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            let active = ACTIVE_TASKS.load(Ordering::SeqCst);
            if active == 0 {
                eprintln!("[etcd-client-py] All tasks completed");
                break;
            }

            if start.elapsed() >= timeout {
                eprintln!(
                    "[etcd-client-py] Timeout waiting for tasks ({} still active)",
                    active
                );
                break;
            }

            eprintln!("[etcd-client-py] Waiting for {} active tasks...", active);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

impl Drop for EtcdRt {
    fn drop(&mut self) {
        eprintln!("[etcd-client-py] Shutting down...");

        // Wait for active tasks to complete (with timeout)
        self.wait_for_tasks(5000);

        eprintln!("[etcd-client-py] Shutdown complete");
    }
}

/// Explicit cleanup function callable from Python
///
/// This can be registered with Python's atexit module for explicit cleanup:
/// ```python
/// import atexit
/// from etcd_client import _cleanup_runtime
/// atexit.register(_cleanup_runtime)
/// ```
#[pyfunction]
pub fn _cleanup_runtime() {
    eprintln!("[etcd-client-py] Explicit cleanup requested");
    if let Some(rt) = RUNTIME.get() {
        // Wait for tasks to complete
        rt.wait_for_tasks(5000);
        eprintln!("[etcd-client-py] Explicit cleanup complete");
    }
}

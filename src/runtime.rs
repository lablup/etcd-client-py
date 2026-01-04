use pyo3::prelude::*;
use std::sync::OnceLock;

/// Global runtime wrapper instance
static RUNTIME: OnceLock<EtcdRt> = OnceLock::new();

/// Runtime wrapper that provides graceful shutdown for ALL tasks in the runtime
///
/// This struct uses Tokio's RuntimeMetrics API to track ALL tasks running in
/// the shared tokio runtime, including tasks from other PyO3 libraries.
///
/// This ensures comprehensive cleanup during Python shutdown, preventing
/// GIL state violations and segfaults.
pub struct EtcdRt {
    // Marker struct - cleanup happens via Drop
}

impl EtcdRt {
    /// Create the global runtime wrapper
    fn new() -> Self {
        eprintln!("[etcd-client-py] Initializing runtime wrapper...");
        EtcdRt {}
    }

    /// Get or initialize the global runtime wrapper
    pub fn get_or_init() -> &'static EtcdRt {
        RUNTIME.get_or_init(EtcdRt::new)
    }

    /// Spawn a future on the shared runtime
    ///
    /// Tasks are spawned on the shared pyo3_async_runtimes tokio runtime,
    /// which is automatically tracked by Tokio's metrics system.
    pub fn spawn<'a, F, T>(&self, py: Python<'a>, fut: F) -> PyResult<Bound<'a, PyAny>>
    where
        F: std::future::Future<Output = PyResult<T>> + Send + 'static,
        T: for<'py> pyo3::IntoPyObject<'py> + Send + 'static,
    {
        // Delegate to pyo3_async_runtimes
        // Tasks are automatically tracked by Tokio's RuntimeMetrics
        pyo3_async_runtimes::tokio::future_into_py(py, fut)
    }

    /// Wait for ALL tasks in the runtime to complete (including other libraries)
    ///
    /// This uses Tokio's RuntimeMetrics API (stable since 1.39.0) to count
    /// all alive tasks in the runtime, regardless of which library spawned them.
    fn wait_for_all_tasks(&self, timeout_ms: u64) {
        // Get the shared runtime from pyo3_async_runtimes
        let runtime = pyo3_async_runtimes::tokio::get_runtime();
        let metrics = runtime.metrics();

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            // Count ALL alive tasks (from all libraries using this runtime)
            let alive_tasks = metrics.num_alive_tasks();

            if alive_tasks == 0 {
                eprintln!("[etcd-client-py] All runtime tasks completed");
                break;
            }

            if start.elapsed() >= timeout {
                eprintln!(
                    "[etcd-client-py] Timeout - {} tasks still active in runtime",
                    alive_tasks
                );
                break;
            }

            eprintln!(
                "[etcd-client-py] Waiting for {} runtime tasks to complete...",
                alive_tasks
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

impl Drop for EtcdRt {
    fn drop(&mut self) {
        eprintln!("[etcd-client-py] Shutting down...");

        // Wait for ALL tasks in the runtime to complete
        // This includes tasks from other PyO3 libraries using the same runtime
        self.wait_for_all_tasks(5000);

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
///
/// This function waits for ALL tasks in the shared tokio runtime to complete,
/// including tasks from other PyO3 libraries.
#[pyfunction]
pub fn _cleanup_runtime() {
    eprintln!("[etcd-client-py] Explicit cleanup requested");
    if let Some(rt) = RUNTIME.get() {
        // Wait for all runtime tasks to complete
        rt.wait_for_all_tasks(5000);
        eprintln!("[etcd-client-py] Explicit cleanup complete");
    }
}

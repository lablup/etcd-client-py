use pyo3::prelude::*;
use std::sync::OnceLock;

/// Global runtime wrapper instance
static RUNTIME: OnceLock<EtcdRt> = OnceLock::new();

/// Runtime wrapper that provides graceful shutdown using pyo3-async-runtimes' shutdown API
///
/// This struct leverages the patched pyo3-async-runtimes library which provides:
/// - Task tracking via task_started() and task_completed()
/// - Explicit shutdown coordination via request_shutdown()
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
    /// Delegates to pyo3_async_runtimes which uses tokio's runtime directly.
    /// Tokio tracks all spawned tasks internally for shutdown.
    pub fn spawn<'a, F, T>(&self, py: Python<'a>, fut: F) -> PyResult<Bound<'a, PyAny>>
    where
        F: std::future::Future<Output = PyResult<T>> + Send + 'static,
        T: for<'py> pyo3::IntoPyObject<'py> + Send + 'static,
    {
        // Delegate to pyo3_async_runtimes - tokio tracks tasks internally
        pyo3_async_runtimes::tokio::future_into_py(py, fut)
    }
}

impl Drop for EtcdRt {
    fn drop(&mut self) {
        eprintln!("[etcd-client-py] Runtime wrapper dropped");
        // Note: Actual runtime shutdown happens via cleanup_runtime() or process exit
    }
}

/// Explicit cleanup function callable from Python
///
/// This should be called at the end of your main async function, before the event loop shuts down:
/// ```python
/// from etcd_client import cleanup_runtime
///
/// async def main():
///     # Your etcd operations here
///     ...
///     # Cleanup before returning
///     cleanup_runtime()
///
/// asyncio.run(main())
/// ```
///
/// This function uses tokio's `shutdown_timeout()` to gracefully shut down all tasks.
#[pyfunction]
pub fn cleanup_runtime() {
    eprintln!("[etcd-client-py] Cleanup requested - using tokio shutdown_timeout");

    // Use tokio's built-in shutdown with 5-second timeout
    let shutdown_performed = pyo3_async_runtimes::tokio::request_shutdown(5000);

    if shutdown_performed {
        eprintln!("[etcd-client-py] Tokio runtime shutdown completed");
    } else {
        eprintln!("[etcd-client-py] Shutdown skipped (borrowed runtime or already shut down)");
    }
}

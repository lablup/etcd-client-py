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

    /// Spawn a future on the shared runtime with task tracking
    ///
    /// Tasks are automatically tracked by calling task_started() before execution
    /// and task_completed() after execution completes.
    pub fn spawn<'a, F, T>(&self, py: Python<'a>, fut: F) -> PyResult<Bound<'a, PyAny>>
    where
        F: std::future::Future<Output = PyResult<T>> + Send + 'static,
        T: for<'py> pyo3::IntoPyObject<'py> + Send + 'static,
    {
        // Increment task counter before spawning
        pyo3_async_runtimes::tokio::task_started();

        // Wrap the future to decrement counter on completion
        let wrapped_fut = async move {
            let result = fut.await;
            pyo3_async_runtimes::tokio::task_completed();
            result
        };

        // Delegate to pyo3_async_runtimes
        pyo3_async_runtimes::tokio::future_into_py(py, wrapped_fut)
    }
}

impl Drop for EtcdRt {
    fn drop(&mut self) {
        eprintln!("[etcd-client-py] Shutting down...");

        // Request shutdown and wait for tasks with 5-second timeout
        let all_completed = pyo3_async_runtimes::tokio::request_shutdown(5000);

        if all_completed {
            eprintln!("[etcd-client-py] All tasks completed");
        } else {
            let remaining = pyo3_async_runtimes::tokio::get_active_task_count();
            eprintln!(
                "[etcd-client-py] Timeout - {} tasks still active",
                remaining
            );
        }

        eprintln!("[etcd-client-py] Shutdown complete");
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
/// This function waits for ALL tracked tasks to complete.
#[pyfunction]
pub fn cleanup_runtime() {
    eprintln!("[etcd-client-py] Explicit cleanup requested");

    // Request shutdown and wait for tasks with 5-second timeout
    let all_completed = pyo3_async_runtimes::tokio::request_shutdown(5000);

    if all_completed {
        eprintln!("[etcd-client-py] All tasks completed");
    } else {
        let remaining = pyo3_async_runtimes::tokio::get_active_task_count();
        eprintln!(
            "[etcd-client-py] Explicit cleanup incomplete - {} tasks still active",
            remaining
        );
    }

    eprintln!("[etcd-client-py] Explicit cleanup complete");
}

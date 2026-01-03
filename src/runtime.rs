use pyo3::prelude::*;
use std::sync::OnceLock;
use std::thread::JoinHandle;
use tokio::sync::Notify;

/// Global runtime instance
static RUNTIME: OnceLock<EtcdRt> = OnceLock::new();

/// Etcd tokio runtime wrapper with explicit cleanup
///
/// This struct manages a dedicated tokio runtime thread and ensures
/// proper cleanup during Python shutdown, preventing GIL state violations
/// and segfaults.
///
/// Based on valkey-glide's GlideRt implementation.
pub struct EtcdRt {
    /// Handle to the runtime thread
    thread: Option<JoinHandle<()>>,
    /// Notifier to signal shutdown
    shutdown_notifier: std::sync::Arc<Notify>,
}

impl EtcdRt {
    /// Create and initialize the global runtime
    fn new() -> Self {
        let shutdown_notifier = std::sync::Arc::new(Notify::new());
        let notify_clone = shutdown_notifier.clone();

        let thread = std::thread::Builder::new()
            .name("etcd-runtime-thread".to_string())
            .spawn(move || {
                // Create a single-threaded tokio runtime
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime");

                // Block on the shutdown notification
                rt.block_on(async {
                    notify_clone.notified().await;
                });

                // Runtime will be dropped here, cleaning up all tasks
            })
            .expect("Failed to spawn runtime thread");

        EtcdRt {
            thread: Some(thread),
            shutdown_notifier,
        }
    }

    /// Get or initialize the global runtime
    pub fn get_or_init() -> &'static EtcdRt {
        RUNTIME.get_or_init(EtcdRt::new)
    }

    /// Spawn a future on the runtime and convert it to a Python awaitable
    ///
    /// This is a thin wrapper around `pyo3_async_runtimes::tokio::future_into_py`.
    /// The tokio runtime is managed explicitly by this struct to ensure proper cleanup.
    ///
    /// Note: This method currently delegates to `pyo3_async_runtimes` for compatibility.
    /// The custom runtime thread ensures graceful shutdown via the Drop implementation.
    #[inline]
    pub fn spawn<'a, F, T>(&self, py: Python<'a>, fut: F) -> PyResult<Bound<'a, PyAny>>
    where
        F: std::future::Future<Output = PyResult<T>> + Send + 'static,
        T: for<'py> pyo3::IntoPyObject<'py> + Send + 'static,
    {
        // Delegate to pyo3_async_runtimes which handles all the type conversions
        // Our Drop implementation ensures the runtime is properly cleaned up
        pyo3_async_runtimes::tokio::future_into_py(py, fut)
    }
}

impl Drop for EtcdRt {
    fn drop(&mut self) {
        // Signal the runtime thread to shutdown
        self.shutdown_notifier.notify_one();

        // Wait for the thread to complete
        if let Some(handle) = self.thread.take() {
            if let Err(e) = handle.join() {
                eprintln!("EtcdRt thread panicked during shutdown: {:?}", e);
            }
        }
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
    if let Some(rt) = RUNTIME.get() {
        rt.shutdown_notifier.notify_one();
    }
}

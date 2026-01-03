use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::JoinHandle;
use tokio::sync::Notify;

/// Global runtime instance
static RUNTIME: OnceLock<EtcdRt> = OnceLock::new();

/// Counter for active tasks (for debugging and graceful shutdown)
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

/// Etcd runtime wrapper with explicit cleanup
///
/// This struct provides task tracking and graceful shutdown during
/// Python shutdown, preventing GIL state violations and segfaults.
pub struct EtcdRt {
    /// Handle to the runtime management thread
    thread: Option<JoinHandle<()>>,
    /// Notifier to signal shutdown
    shutdown_notifier: Arc<Notify>,
}

impl EtcdRt {
    /// Create and initialize the global runtime wrapper
    fn new() -> Self {
        eprintln!("[etcd-client-py] Initializing runtime wrapper...");

        let shutdown_notifier = Arc::new(Notify::new());
        let notify_clone = shutdown_notifier.clone();

        // Spawn a management thread for cleanup coordination
        let thread = std::thread::Builder::new()
            .name("etcd-runtime-manager".to_string())
            .spawn(move || {
                let mgmt_rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create management runtime");

                mgmt_rt.block_on(async {
                    notify_clone.notified().await;
                });
            })
            .expect("Failed to spawn management thread");

        eprintln!("[etcd-client-py] Runtime wrapper initialized");

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
    /// This uses pyo3_async_runtimes for Python future integration while
    /// tracking active tasks for graceful shutdown.
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
                    "[etcd-client-py] Timeout waiting for tasks ({}  still active)",
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
        eprintln!("[etcd-client-py] Shutting down tokio runtime...");

        // Wait for active tasks to complete (with timeout)
        self.wait_for_tasks(5000);

        // Signal the management thread to shutdown
        self.shutdown_notifier.notify_one();

        // Wait for the management thread
        if let Some(handle) = self.thread.take() {
            if let Err(e) = handle.join() {
                eprintln!(
                    "[etcd-client-py] Management thread panicked during shutdown: {:?}",
                    e
                );
            }
        }

        eprintln!("[etcd-client-py] Runtime shutdown complete (tasks waited)");
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

        // Signal shutdown
        rt.shutdown_notifier.notify_one();

        eprintln!("[etcd-client-py] Explicit cleanup complete (tasks waited)");
    }
}

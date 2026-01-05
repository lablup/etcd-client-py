use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Count of active client context managers (those currently inside `async with`).
static ACTIVE_CONTEXTS: AtomicUsize = AtomicUsize::new(0);

/// Called when a client enters its async context (`__aenter__`).
///
/// Increments the active context count. If this is the first context after
/// a previous shutdown, the tokio runtime will be lazily re-initialized
/// on the first spawn operation.
pub fn enter_context() {
    ACTIVE_CONTEXTS.fetch_add(1, Ordering::SeqCst);
}

/// Called when a client exits its async context (`__aexit__`).
///
/// Decrements the active context count. If this was the last active context
/// (count drops from 1 to 0), automatically triggers runtime shutdown.
///
/// Returns `true` if cleanup was triggered, `false` otherwise.
pub fn exit_context() -> bool {
    let prev = ACTIVE_CONTEXTS.fetch_sub(1, Ordering::SeqCst);

    if prev == 1 {
        // Was 1, now 0 - last context exited, cleanup runtime
        pyo3_async_runtimes::tokio::request_shutdown(5000);
        true
    } else {
        false
    }
}

/// Get the current count of active client contexts.
///
/// Useful for debugging and testing automatic cleanup behavior.
/// Returns 0 when no clients are in an active context manager.
#[pyfunction]
pub fn active_context_count() -> usize {
    ACTIVE_CONTEXTS.load(Ordering::SeqCst)
}

/// Explicitly request graceful shutdown of the tokio runtime.
///
/// In most cases, the runtime is automatically cleaned up when the last
/// client context exits. This function is provided for cases where explicit
/// control is needed.
///
/// # Example
///
/// ```python
/// import asyncio
/// from etcd_client import cleanup_runtime
///
/// async def main():
///     # Your etcd operations here
///     ...
///     # Explicit cleanup (usually not needed)
///     cleanup_runtime()
///
/// asyncio.run(main())
/// ```
///
/// This function uses tokio's `shutdown_timeout()` to gracefully shut down all tasks,
/// waiting up to 5 seconds for pending tasks to complete.
#[pyfunction]
pub fn cleanup_runtime() {
    pyo3_async_runtimes::tokio::request_shutdown(5000);
}

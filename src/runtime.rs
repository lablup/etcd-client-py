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
/// Decrements the active context count and returns `true` if this was the
/// last active context (count drops from 1 to 0).
///
/// Note: This function does NOT trigger the shutdown directly. The caller
/// should trigger shutdown from Python AFTER the tokio task completes, to
/// avoid a race condition where the runtime starts shutting down while the
/// task is still returning its result.
///
/// Returns `true` if this was the last context, `false` otherwise.
pub fn exit_context() -> bool {
    let prev = ACTIVE_CONTEXTS.fetch_sub(1, Ordering::SeqCst);
    prev == 1
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

/// Internal function to trigger runtime shutdown in the background.
///
/// This is called from Python AFTER the tokio task has completed and
/// returned its result. This avoids a race condition where the runtime
/// starts shutting down while a task is still trying to return.
///
/// Users should not call this directly - it's used internally by the
/// client's `__aexit__` implementation.
#[pyfunction]
pub fn _trigger_shutdown() {
    pyo3_async_runtimes::tokio::request_shutdown_background(5000);
}

/// Internal function to join the pending runtime shutdown thread.
///
/// This is called from Python via `asyncio.to_thread()` to block until
/// the runtime thread has fully terminated. The GIL is released during
/// the blocking wait.
///
/// Users should not call this directly - it's used internally by the
/// client's `__aexit__` implementation.
#[pyfunction]
pub fn _join_pending_shutdown(py: Python<'_>) -> bool {
    pyo3_async_runtimes::tokio::join_pending_shutdown(py)
}

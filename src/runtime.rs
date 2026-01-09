use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Shutdown timeout in milliseconds for graceful runtime cleanup.
const SHUTDOWN_TIMEOUT_MS: u64 = 5000;

/// Active client context count.
static ACTIVE_CONTEXTS: AtomicUsize = AtomicUsize::new(0);

// ============================================================================
// Context Counting (internal)
// ============================================================================

/// Increment context count on `__aenter__`.
pub(crate) fn enter_context() {
    ACTIVE_CONTEXTS.fetch_add(1, Ordering::SeqCst);
}

/// Decrement context count on `__aexit__`.
/// Returns `true` if this was the last context (count dropped to 0).
pub(crate) fn exit_context() -> bool {
    let prev = ACTIVE_CONTEXTS.fetch_sub(1, Ordering::SeqCst);
    prev == 1
}

// ============================================================================
// Public API
// ============================================================================

/// Get current active context count (for debugging/testing).
#[pyfunction]
pub fn active_context_count() -> usize {
    ACTIVE_CONTEXTS.load(Ordering::SeqCst)
}

/// Explicitly request graceful runtime shutdown.
///
/// Usually not needed - runtime is automatically cleaned up when the last
/// client context exits.
#[pyfunction]
pub fn cleanup_runtime() {
    pyo3_async_runtimes::tokio::request_shutdown(SHUTDOWN_TIMEOUT_MS);
}

// ============================================================================
// Internal Shutdown Helpers (used by __aexit__)
// ============================================================================

/// Trigger runtime shutdown in background. Called from Python after tokio task completes.
#[pyfunction]
pub fn _trigger_shutdown() {
    pyo3_async_runtimes::tokio::request_shutdown_background(SHUTDOWN_TIMEOUT_MS);
}

/// Block until runtime thread terminates. Called via `asyncio.to_thread()`.
#[pyfunction]
pub fn _join_pending_shutdown(py: Python<'_>) -> bool {
    pyo3_async_runtimes::tokio::join_pending_shutdown(py)
}

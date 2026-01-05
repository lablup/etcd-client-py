use pyo3::prelude::*;

/// Request graceful shutdown of the tokio runtime.
///
/// This should be called at the end of your main async function, before the event loop shuts down:
///
/// ```python
/// import asyncio
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
/// This function uses tokio's `shutdown_timeout()` to gracefully shut down all tasks,
/// waiting up to 5 seconds for pending tasks to complete.
#[pyfunction]
pub fn cleanup_runtime() {
    pyo3_async_runtimes::tokio::request_shutdown(5000);
}

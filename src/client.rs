use etcd_client::{Client as EtcdClient, ConnectOptions};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3_async_runtimes::tokio::future_into_py;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::communicator::PyCommunicator;
use crate::error::PyClientError;
use crate::lock_manager::{EtcdLockManager, PyEtcdLockOption};

/// Python wrapper coroutine for async exit.
///
/// Shutdown sequence:
/// 1. Await inner_cleanup (tokio task) - returns true if this was the last context
/// 2. If last context: trigger_shutdown_fn() signals runtime to shut down
/// 3. Then await to_thread(join_fn) to block until runtime thread terminates
const AEXIT_WRAPPER_CODE: &std::ffi::CStr = c"
async def _aexit_wrapper():
    is_last = await inner_cleanup
    if is_last:
        trigger_shutdown_fn()
        await to_thread(join_fn)
_result = _aexit_wrapper()
";

#[pyclass(name = "ConnectOptions")]
#[derive(Debug, Clone, Default)]
pub struct PyConnectOptions(pub ConnectOptions);

#[pymethods]
impl PyConnectOptions {
    #[new]
    fn new() -> Self {
        Self(ConnectOptions::new())
    }

    fn with_user(&self, name: String, password: String) -> Self {
        PyConnectOptions(self.0.clone().with_user(name, password))
    }

    fn with_keep_alive(&self, interval: f64, timeout: f64) -> Self {
        PyConnectOptions(self.0.clone().with_keep_alive(
            Duration::from_secs_f64(interval),
            Duration::from_secs_f64(timeout),
        ))
    }

    fn with_keep_alive_while_idle(&self, enabled: bool) -> Self {
        PyConnectOptions(self.0.clone().with_keep_alive_while_idle(enabled))
    }

    fn with_connect_timeout(&self, connect_timeout: f64) -> Self {
        PyConnectOptions(
            self.0
                .clone()
                .with_connect_timeout(Duration::from_secs_f64(connect_timeout)),
        )
    }

    fn with_timeout(&self, timeout: f64) -> Self {
        PyConnectOptions(
            self.0
                .clone()
                .with_timeout(Duration::from_secs_f64(timeout)),
        )
    }

    fn with_tcp_keepalive(&self, tcp_keepalive: f64) -> Self {
        PyConnectOptions(
            self.0
                .clone()
                .with_tcp_keepalive(Duration::from_secs_f64(tcp_keepalive)),
        )
    }

    // TODO: Implement "tls", "tls-openssl" authentification
}

#[pyclass(name = "Client")]
#[derive(Clone)]
pub struct PyClient {
    pub endpoints: Vec<String>,
    pub connect_options: PyConnectOptions,
    pub lock_options: Option<PyEtcdLockOption>,
    pub lock_manager: Option<Arc<Mutex<EtcdLockManager>>>,
}

#[pymethods]
impl PyClient {
    #[new]
    #[pyo3(signature = (endpoints, connect_options=None, lock_options=None))]
    fn new(
        endpoints: Vec<String>,
        connect_options: Option<PyConnectOptions>,
        lock_options: Option<PyEtcdLockOption>,
    ) -> Self {
        Self {
            endpoints,
            connect_options: connect_options.unwrap_or_default(),
            lock_options,
            lock_manager: None,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Client(endpoints={:?}, connect_options={:?}, lock_options={:?})",
            self.endpoints, self.connect_options, self.lock_options
        )
    }

    #[pyo3(signature = (connect_options=None))]
    pub fn connect(&self, connect_options: Option<PyConnectOptions>) -> Self {
        let mut result = self.clone();
        result.connect_options = connect_options.unwrap_or(self.connect_options.clone());
        result
    }

    #[pyo3(signature = (lock_options, connect_options=None))]
    pub fn with_lock(
        &self,
        lock_options: PyEtcdLockOption,
        connect_options: Option<PyConnectOptions>,
    ) -> Self {
        let mut result = self.clone();
        result.connect_options = connect_options.unwrap_or(self.connect_options.clone());
        result.lock_options = Some(lock_options);
        result
    }

    #[pyo3(signature = ())]
    fn __aenter__<'a>(&'a mut self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        crate::runtime::enter_context();

        let endpoints = self.endpoints.clone();
        let connect_options = self.connect_options.clone();
        let lock_options = self.lock_options.clone();

        let lock_manager = if let Some(ref lock_options) = lock_options {
            self.lock_manager = Some(Arc::new(Mutex::new(EtcdLockManager::new(
                self.clone(),
                lock_options.clone(),
            ))));
            Some(self.lock_manager.clone().unwrap())
        } else {
            None
        };

        future_into_py(py, async move {
            match EtcdClient::connect(endpoints, Some(connect_options.0)).await {
                Ok(client) => {
                    if let Some(lock_manager) = lock_manager {
                        Ok(lock_manager.lock().await.handle_aenter().await?)
                    } else {
                        Ok(PyCommunicator::new(client))
                    }
                }
                Err(e) => {
                    crate::runtime::exit_context();
                    Err(PyClientError(e).into())
                }
            }
        })
    }

    #[pyo3(signature = (*_args))]
    fn __aexit__<'a>(
        &'a self,
        py: Python<'a>,
        _args: &Bound<'a, PyTuple>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let lock_manager = self
            .lock_options
            .as_ref()
            .map(|_| self.lock_manager.clone().unwrap());

        // Tokio task: cleanup and return whether this was the last context
        let inner_cleanup = future_into_py(py, async move {
            if let Some(lock_manager) = lock_manager {
                lock_manager.lock().await.handle_aexit().await?;
            }
            Ok(crate::runtime::exit_context())
        })?;

        // Build Python wrapper coroutine
        let etcd_client = py.import("etcd_client")?;
        let asyncio = py.import("asyncio")?;

        let globals = PyDict::new(py);
        globals.set_item("inner_cleanup", inner_cleanup)?;
        globals.set_item("trigger_shutdown_fn", etcd_client.getattr("_trigger_shutdown")?)?;
        globals.set_item("join_fn", etcd_client.getattr("_join_pending_shutdown")?)?;
        globals.set_item("to_thread", asyncio.getattr("to_thread")?)?;

        py.run(AEXIT_WRAPPER_CODE, Some(&globals), None)?;

        Ok(globals.get_item("_result")?.unwrap())
    }
}

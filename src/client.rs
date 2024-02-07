use etcd_client::{Client as EtcdClient, ConnectOptions};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::communicator::PyCommunicator;
use crate::error::PyClientError;
use crate::lock_manager::{EtcdLockManager, PyEtcdLockOption};

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
    fn new(
        endpoints: Vec<String>,
        connect_options: Option<PyConnectOptions>,
        lock_options: Option<PyEtcdLockOption>,
    ) -> Self {
        let connect_options = connect_options.unwrap_or(PyConnectOptions::default());
        Self {
            endpoints,
            connect_options,
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

    pub fn connect(&self, connect_options: Option<PyConnectOptions>) -> Self {
        let mut result = self.clone();
        result.connect_options = connect_options.unwrap_or(self.connect_options.clone());
        result
    }

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

    fn __aenter__<'a>(&'a mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
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
                Err(e) => Err(PyClientError(e).into()),
            }
        })
    }

    #[pyo3(signature = (*_args))]
    fn __aexit__<'a>(&'a self, py: Python<'a>, _args: &PyTuple) -> PyResult<&'a PyAny> {
        let lock_options = self.lock_options.clone();

        let lock_manager = if lock_options.is_some() {
            Some(self.lock_manager.clone().unwrap())
        } else {
            None
        };

        future_into_py(py, async move {
            if let Some(lock_manager) = lock_manager {
                return lock_manager.lock().await.handle_aexit().await;
            }
            Ok(())
        })
    }
}

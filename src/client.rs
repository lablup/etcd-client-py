use etcd_client::{Client as EtcdClient, ConnectOptions};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::communicator::PyCommunicator;
use crate::error::PyClientError;

#[pyclass(name = "ConnectOptions")]
#[derive(Clone, Default)]
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
    endpoints: Vec<String>,
    options: PyConnectOptions,
}

#[pymethods]
impl PyClient {
    #[new]
    fn new(endpoints: Vec<String>, options: Option<PyConnectOptions>) -> Self {
        let options = options.unwrap_or(PyConnectOptions::default());
        Self { endpoints, options }
    }

    fn connect(&self, options: Option<PyConnectOptions>) -> Self {
        let mut result = self.clone();
        result.options = options.unwrap_or(self.options.clone());
        result
    }

    fn __aenter__<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let endpoints = self.endpoints.clone();
        let options = self.options.clone();
        future_into_py(py, async move {
            let result = EtcdClient::connect(endpoints, Some(options.0)).await;
            result
                .map(|client| PyCommunicator(Arc::new(Mutex::new(client))))
                .map_err(|e| PyClientError(e).into())
        })
    }

    #[pyo3(signature = (*_args))]
    fn __aexit__<'a>(&'a self, py: Python<'a>, _args: &PyTuple) -> PyResult<&'a PyAny> {
        future_into_py(py, async move { Ok(()) })
    }
}

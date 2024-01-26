use etcd_client::Client as EtcdClient;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::communicator::PyCommunicator;
use crate::error::Error;

#[pyclass(name = "Client")]
#[derive(Clone)]
pub struct PyClient {
    endpoints: Vec<String>,
}

#[pymethods]
impl PyClient {
    #[new]
    fn new(endpoints: Vec<String>) -> Self {
        Self { endpoints }
    }

    fn connect(&self) -> Self {
        self.clone()
    }

    fn __aenter__<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let endpoints = self.endpoints.clone();
        future_into_py(py, async move {
            let result = EtcdClient::connect(endpoints, None).await;
            result
                .map(|client| PyCommunicator(Arc::new(Mutex::new(client))))
                .map_err(|e| Error(e).into())
        })
    }

    #[pyo3(signature = (*_args))]
    fn __aexit__<'a>(&'a self, py: Python<'a>, _args: &PyTuple) -> PyResult<&'a PyAny> {
        future_into_py(py, async move { Ok(()) })
    }
}

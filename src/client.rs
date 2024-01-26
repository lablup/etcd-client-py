use etcd_client::Client as RustClient;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::communicator::Communicator;
use crate::error::Error;

#[pyclass]
#[derive(Clone)]
pub struct Client {
    endpoints: Vec<String>,
}

#[pymethods]
impl Client {
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
            let result = RustClient::connect(endpoints, None).await;
            result
                .map(|client| Communicator(Arc::new(Mutex::new(client))))
                .map_err(|e| Error(e).into())
        })
    }

    #[pyo3(signature = (*_args))]
    fn __aexit__<'a>(&'a self, py: Python<'a>, _args: &PyTuple) -> PyResult<&'a PyAny> {
        future_into_py(py, async move { Ok(()) })
    }
}

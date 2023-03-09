use std::collections::HashMap;
use std::sync::Arc;

use etcd_client::Client as RustClient;
use etcd_client::Error as RustError;
use etcd_client::{DeleteOptions, GetOptions};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_asyncio::tokio::future_into_py;
use tokio::sync::Mutex;

create_exception!(etcd_client, ClientError, PyException);

struct Error(RustError);

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        ClientError::new_err(error.0.to_string())
    }
}

#[pyclass]
#[derive(Clone)]
struct Client {
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
                .map(|client| {
                    Communicator(Arc::new(Mutex::new(client)))
                })
                .map_err(|e| Error(e).into())
        })
    }

    #[pyo3(signature = (*_args))]
    fn __aexit__<'a>(&'a self, py: Python<'a>, _args: &PyTuple) -> PyResult<&'a PyAny> {
        future_into_py(py, async move {
            Ok(())
        })
    }
}

#[pyclass]
struct Communicator(Arc<Mutex<RustClient>>);

#[pymethods]
impl Communicator {
    fn get<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.get(key, None).await;
            result
                .map(|response| {
                    let kvs = response.kvs();
                    if !kvs.is_empty() {
                        Some(String::from_utf8(kvs[0].value().to_owned()).unwrap())
                    } else {
                        None
                    }
                })
                .map_err(|e| Error(e).into())
        })
    }

    fn get_prefix<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = GetOptions::new().with_prefix();
            let result = client.get(key, Some(options)).await;
            result
                .map(|response| {
                    let mut result = HashMap::new();
                    let kvs = response.kvs();
                    for kv in kvs {
                        let key = String::from_utf8(kv.key().to_owned()).unwrap();
                        let value = String::from_utf8(kv.value().to_owned()).unwrap();
                        result.insert(key, value);
                    }
                    result
                })
                .map_err(|e| Error(e).into())
        })
    }

    fn keys_prefix<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = GetOptions::new().with_prefix();
            let result = client.get(key, Some(options)).await;
            result
                .map(|response| {
                    let mut result = Vec::new();
                    let kvs = response.kvs();
                    for kv in kvs {
                        let key = String::from_utf8(kv.key().to_owned()).unwrap();
                        result.push(key);
                    }
                    result
                })
                .map_err(|e| Error(e).into())
        })
    }

    fn delete<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.delete(key, None).await;
            result
                .map(|_| ())
                .map_err(|e| Error(e).into())
        })
    }

    fn delete_prefix<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = DeleteOptions::new().with_prefix();
            let result = client.delete(key, Some(options)).await;
            result
                .map(|_| ())
                .map_err(|e| Error(e).into())
        })
    }

    fn put<'a>(&'a self, py: Python<'a>, key: String, value: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.put(key, value, None).await;
            result
                .map(|_| ())
                .map_err(|e| Error(e).into())
        })
    }
}

#[pymodule]
#[pyo3(name = "etcd_client")]
fn init(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<Client>()?;
    Ok(())
}

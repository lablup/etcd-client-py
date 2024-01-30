use etcd_client::{Client as EtcdClient, PutOptions};
use etcd_client::{DeleteOptions, GetOptions, WatchOptions};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::condvar::PyCondVar;
use crate::error::PyClientError;
use crate::txn::PyTxn;
use crate::txn_response::PyTxnResponse;
use crate::watch::PyWatch;

#[pyclass(name = "Communicator")]
pub struct PyCommunicator(pub Arc<Mutex<EtcdClient>>);

#[pymethods]
impl PyCommunicator {
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
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn get_prefix<'a>(&'a self, py: Python<'a>, prefix: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = GetOptions::new().with_prefix();
            let result = client.get(prefix, Some(options)).await;
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
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn put<'a>(&'a self, py: Python<'a>, key: String, value: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.put(key, value, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn put_prefix<'a>(
        &'a self,
        py: Python<'a>,
        prefix: String,
        value: String,
    ) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = PutOptions::new().with_prev_key();
            let result = client.put(prefix, value, Some(options)).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn delete<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;

            let result = client.delete(key, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn delete_prefix<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = DeleteOptions::new().with_prefix();
            let result = client.delete(key, Some(options)).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn txn<'a>(&'a self, py: Python<'a>, txn: PyTxn) -> PyResult<&'a PyAny> {
        let client = self.0.clone();

        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.txn(txn.0).await;
            result
                .map(PyTxnResponse)
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn replace<'a>(
        &'a self,
        py: Python<'a>,
        key: String,
        initial_val: String,
        new_val: String,
    ) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            match client.get(key.clone(), None).await {
                Ok(response) => {
                    if let Some(key_value) = response.kvs().get(0) {
                        if *key_value.value_str().unwrap() == initial_val {
                            match client.put(key, new_val, None).await {
                                Ok(_) => Ok(true), // replace successful
                                Err(e) => Err(PyClientError(e)),
                            }
                        } else {
                            Ok(false) // initial_val not equal to current value
                        }
                    } else {
                        Ok(false) // Key does not exist
                    }
                }
                Err(e) => Err(PyClientError(e)),
            }
            .map_err(|e| PyErr::new::<PyException, _>(format!("{}", e.0)))
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
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn watch(
        &self,
        key: String,
        once: Option<bool>,
        ready_event: Option<PyCondVar>,
        cleanup_event: Option<PyCondVar>,
    ) -> PyWatch {
        let client = self.0.clone();
        let once = once.unwrap_or(false);
        PyWatch::new(client, key, once, None, ready_event, cleanup_event)
    }

    fn watch_prefix(
        &self,
        key: String,
        once: Option<bool>,
        ready_event: Option<PyCondVar>,
        cleanup_event: Option<PyCondVar>,
    ) -> PyWatch {
        let client = self.0.clone();
        let once = once.unwrap_or(false);
        let options = WatchOptions::new().with_prefix();
        PyWatch::new(client, key, once, Some(options), ready_event, cleanup_event)
    }
}

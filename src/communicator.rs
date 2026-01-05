use etcd_client::Client as EtcdClient;
use etcd_client::{DeleteOptions, GetOptions, WatchOptions};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
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
    // TODO: Implement and use the CRUD response types

    fn get<'a>(&'a self, py: Python<'a>, key: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.get(key, None).await;
            result
                .map(|response| {
                    let kvs = response.kvs();
                    if !kvs.is_empty() {
                        Some(kvs[0].value().to_owned())
                    } else {
                        None
                    }
                })
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn get_prefix<'a>(&'a self, py: Python<'a>, prefix: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = GetOptions::new().with_prefix();
            let result = client.get(prefix, Some(options)).await;
            result
                .map(|response| {
                    let mut list = vec![];
                    let kvs = response.kvs();
                    for kv in kvs {
                        list.push((kv.key().to_owned(), kv.value().to_owned()));
                    }
                    list
                })
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn put<'a>(
        &'a self,
        py: Python<'a>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.put(key, value, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn delete<'a>(&'a self, py: Python<'a>, key: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.delete(key, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn delete_prefix<'a>(&'a self, py: Python<'a>, key: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = DeleteOptions::new().with_prefix();
            let result = client.delete(key, Some(options)).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn txn<'a>(&'a self, py: Python<'a>, txn: PyTxn) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.txn(txn.0).await;
            result
                .map(PyTxnResponse)
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn keys_prefix<'a>(&'a self, py: Python<'a>, key: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
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
                        result.push(kv.key().to_owned());
                    }
                    result
                })
                .map_err(|e| PyClientError(e).into())
        })
    }

    fn lock<'a>(&'a self, py: Python<'a>, name: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.lock(name, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn unlock<'a>(&'a self, py: Python<'a>, name: Vec<u8>) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.unlock(name).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn lease_grant<'a>(&'a self, py: Python<'a>, ttl: i64) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.lease_grant(ttl, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn lease_revoke<'a>(&'a self, py: Python<'a>, id: i64) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.lease_revoke(id).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn lease_time_to_live<'a>(&'a self, py: Python<'a>, id: i64) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.lease_time_to_live(id, None).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    fn lease_keep_alive<'a>(&'a self, py: Python<'a>, id: i64) -> PyResult<Bound<'a, PyAny>> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.lease_keep_alive(id).await;
            result.map(|_| ()).map_err(|e| PyClientError(e).into())
        })
    }

    #[pyo3(signature = (key, once=None, ready_event=None, cleanup_event=None))]
    fn watch(
        &self,
        key: Vec<u8>,
        once: Option<bool>,
        ready_event: Option<PyCondVar>,
        cleanup_event: Option<PyCondVar>,
    ) -> PyWatch {
        let client = self.0.clone();
        let once = once.unwrap_or(false);
        PyWatch::new(client, key, once, None, ready_event, cleanup_event)
    }

    #[pyo3(signature = (key, once=None, ready_event=None, cleanup_event=None))]
    fn watch_prefix(
        &self,
        key: Vec<u8>,
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

impl PyCommunicator {
    pub fn new(client: EtcdClient) -> PyCommunicator {
        PyCommunicator(Arc::new(Mutex::new(client)))
    }
}

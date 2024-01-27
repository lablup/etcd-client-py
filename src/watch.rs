use etcd_client::Client as EtcdClient;
use etcd_client::WatchOptions;
use pyo3::exceptions::PyStopAsyncIteration;
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::condvar::PyCondVar;
use crate::error::Error;
use crate::stream::Stream;

#[pyclass(name = "Watch")]
#[derive(Clone)]
pub struct PyWatch {
    client: Arc<Mutex<EtcdClient>>,
    key: String,
    options: Option<WatchOptions>,
    stream: Option<Arc<Mutex<Stream>>>,
    ready_event: Option<PyCondVar>,
    cleanup_event: Option<PyCondVar>,
}

impl PyWatch {
    pub fn new(
        client: Arc<Mutex<EtcdClient>>,
        key: String,
        options: Option<WatchOptions>,
        ready_event: Option<PyCondVar>,
        cleanup_event: Option<PyCondVar>,
    ) -> Self {
        Self {
            client,
            key,
            options,
            stream: None,
            ready_event,
            cleanup_event,
        }
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        if !self.stream.is_none() {
            return Ok(());
        }
        let mut client = self.client.lock().await;
        let key = self.key.clone();
        let options = self.options.clone();
        let result = client.watch(key, options).await;
        result
            .map(|(_, stream)| {
                self.stream = Some(Arc::new(Mutex::new(Stream::new(stream))));
                ()
            })
            .map_err(|e| Error(e))
    }
}

#[pymethods]
impl PyWatch {
    fn __aiter__(&self) -> Self {
        self.clone()
    }

    fn __anext__<'a>(&'a mut self, py: Python<'a>) -> PyResult<Option<PyObject>> {
        let watch = Arc::new(Mutex::new(self.clone()));
        let result = future_into_py(py, async move {
            let mut watch = watch.lock().await;
            watch.init().await?;
            let stream = watch.stream.as_ref().unwrap().clone();
            let mut stream = stream.lock().await;
            let option = stream.next().await;

            let result = match option {
                Some(result) => result,
                None => return Err(PyStopAsyncIteration::new_err(())),
            };
            Ok(result?)
        });

        if self.ready_event.is_some() {
            self.ready_event.as_ref().unwrap().notify_all(py)?;
        }

        Ok(Some(result.unwrap().into()))
    }
}

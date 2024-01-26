use etcd_client::Client as RustClient;
use etcd_client::WatchOptions;
use pyo3::exceptions::PyStopAsyncIteration;
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::Error;
use crate::stream::Stream;

#[pyclass]
#[derive(Clone)]
pub struct Watch {
    client: Arc<Mutex<RustClient>>,
    key: String,
    options: Option<WatchOptions>,
    stream: Option<Arc<Mutex<Stream>>>,
}

impl Watch {
    pub fn new(client: Arc<Mutex<RustClient>>, key: String, options: Option<WatchOptions>) -> Self {
        Self {
            client,
            key,
            options,
            stream: None,
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
impl Watch {
    fn __aiter__(&self) -> Self {
        self.clone()
    }

    fn __anext__<'a>(&'a self, py: Python<'a>) -> Option<PyObject> {
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
        Some(result.unwrap().into())
    }
}

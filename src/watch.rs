use etcd_client::Client as EtcdClient;
use etcd_client::WatchOptions;
use etcd_client::Watcher;
use pyo3::exceptions::PyStopAsyncIteration;
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;

use crate::condvar::PyCondVar;
use crate::error::PyClientError;
use crate::watch_event_stream::PyWatchEventStream;

#[pyclass(name = "Watch")]
#[derive(Clone)]
pub struct PyWatch {
    client: Arc<Mutex<EtcdClient>>,
    key: String,
    once: bool,
    options: Option<WatchOptions>,
    watcher: Arc<Mutex<Option<Watcher>>>,
    event_stream_init_notifier: Arc<Notify>,
    event_stream: Arc<Mutex<Option<PyWatchEventStream>>>,
    ready_event: Option<PyCondVar>,
    #[allow(dead_code)]
    cleanup_event: Option<PyCondVar>,
}

impl PyWatch {
    pub fn new(
        client: Arc<Mutex<EtcdClient>>,
        key: String,
        once: bool,
        options: Option<WatchOptions>,
        ready_event: Option<PyCondVar>,
        cleanup_event: Option<PyCondVar>,
    ) -> Self {
        Self {
            client,
            key,
            once,
            options,
            event_stream_init_notifier: Arc::new(Notify::new()),
            event_stream: Arc::new(Mutex::new(None)),
            watcher: Arc::new(Mutex::new(None)),
            ready_event,
            cleanup_event,
        }
    }

    pub async fn init(&mut self) -> Result<(), PyClientError> {
        // Already initialized
        let mut event_stream = self.event_stream.lock().await;
        if event_stream.is_some() {
            return Ok(());
        }

        let event_stream_init_notifier = self.event_stream_init_notifier.clone();

        let mut client = self.client.lock().await;

        match client.watch(self.key.clone(), self.options.clone()).await {
            Ok((watcher, stream)) => {
                *event_stream = Some(PyWatchEventStream::new(stream, self.once));
                *self.watcher.lock().await = Some(watcher);

                event_stream_init_notifier.notify_waiters();

                if let Some(ready_event) = &self.ready_event {
                    ready_event._notify_waiters().await;
                }
                Ok(())
            }
            Err(error) => Err(PyClientError(error)),
        }
    }
}

#[pymethods]
impl PyWatch {
    fn __aiter__(&self) -> Self {
        self.clone()
    }

    fn __anext__<'a>(&'a mut self, py: Python<'a>) -> PyResult<Option<PyObject>> {
        let watch = Arc::new(Mutex::new(self.clone()));
        let event_stream_init_notifier = self.event_stream_init_notifier.clone();
        let watcher = self.watcher.clone();
        let once = self.once;

        Ok(Some(
            future_into_py(py, async move {
                let mut watch = watch.lock().await;
                watch.init().await?;

                let mut event_stream = watch.event_stream.lock().await;

                if event_stream.is_none() {
                    event_stream_init_notifier.notified().await;
                }

                let event_stream = event_stream.as_mut().unwrap();

                let event = match event_stream.next().await {
                    Some(result) => {
                        if once {
                            let mut watcher = watcher.lock().await;
                            watcher.as_mut().unwrap().cancel().await.unwrap();
                        }

                        result
                    }
                    None => return Err(PyStopAsyncIteration::new_err(())),
                }?;

                Ok(event)
            })?
            .into(),
        ))
    }
}

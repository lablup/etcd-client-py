use etcd_client::Client as EtcdClient;
use etcd_client::Error as EtcdError;
use etcd_client::WatchOptions;
use etcd_client::WatchRequestSender;
use pyo3::exceptions::PyStopAsyncIteration;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;

use crate::condvar::PyCondVar;
use crate::error::PyClientError;
use crate::watch_event_stream::PyWatchEventStream;

struct CreatedWatch {
    watch_id: i64,
    request_sender: WatchRequestSender,
}

#[pyclass(name = "Watch")]
#[derive(Clone)]
pub struct PyWatch {
    client: Arc<Mutex<EtcdClient>>,
    key: Vec<u8>,
    once: bool,
    options: Option<WatchOptions>,
    created_watch: Arc<Mutex<Option<CreatedWatch>>>,
    event_stream_init_notifier: Arc<Notify>,
    event_stream: Arc<Mutex<Option<PyWatchEventStream>>>,
    ready_event: Option<PyCondVar>,
    #[allow(dead_code)]
    cleanup_event: Option<PyCondVar>,
}

impl PyWatch {
    pub fn new(
        client: Arc<Mutex<EtcdClient>>,
        key: Vec<u8>,
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
            created_watch: Arc::new(Mutex::new(None)),
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

        let (request_sender, mut response_stream) = client
            .watch(self.key.clone(), self.options.clone())
            .await
            .map_err(PyClientError)?
            .split();

        // `Client::watch` returns before the server creates the watch, and `ready_event` promises
        // that it has been created.
        let watch_id = match response_stream.message().await.map_err(PyClientError)? {
            Some(response) if response.created() => response.watch_id(),
            _ => {
                return Err(PyClientError(EtcdError::WatchError(
                    "failed to create watch".to_string(),
                )));
            }
        };

        *event_stream = Some(PyWatchEventStream::new(response_stream, self.once));
        *self.created_watch.lock().await = Some(CreatedWatch {
            watch_id,
            request_sender,
        });

        event_stream_init_notifier.notify_waiters();

        if let Some(ready_event) = &self.ready_event {
            ready_event._notify_waiters().await;
        }
        Ok(())
    }
}

#[pymethods]
impl PyWatch {
    fn __aiter__(&self) -> Self {
        self.clone()
    }

    fn __anext__<'a>(&'a mut self, py: Python<'a>) -> PyResult<Option<Py<PyAny>>> {
        let watch = Arc::new(Mutex::new(self.clone()));
        let event_stream_init_notifier = self.event_stream_init_notifier.clone();
        let created_watch = self.created_watch.clone();
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
                            let mut created_watch = created_watch.lock().await;
                            let created_watch = created_watch.as_mut().unwrap();
                            created_watch
                                .request_sender
                                .cancel(created_watch.watch_id)
                                .await
                                .unwrap();
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

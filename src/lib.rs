use std::collections::HashMap;
use std::sync::Arc;

use etcd_client::Client as RustClient;
use etcd_client::Error as RustError;
use etcd_client::Event as RustEvent;
use etcd_client::EventType as RustEventType;
use etcd_client::WatchStream as RustStream;
use etcd_client::{DeleteOptions, GetOptions, WatchOptions};
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyStopAsyncIteration};
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyTuple;
use pyo3_asyncio::tokio::future_into_py;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

create_exception!(etcd_client, ClientError, PyException);

struct Error(RustError);

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        ClientError::new_err(error.0.to_string())
    }
}

#[pyclass]
#[derive(PartialEq, Eq, Clone)]
struct EventType(RustEventType);

#[pymethods]
impl EventType {
    #[classattr]
    const PUT: Self = Self(RustEventType::Put);

    #[classattr]
    const DELETE: Self = Self(RustEventType::Delete);
}

#[pyclass]
#[derive(PartialEq, Eq, Clone)]
struct Event {
    key: String,
    value: String,
    prev_value: Option<String>,
    event: EventType,
}

#[pymethods]
impl Event {
    #[new]
    #[pyo3(signature = (key, value, prev_value, event))]
    fn new(key: String, value: String, prev_value: Option<String>, event: EventType) -> Self {
        Self { key, value, prev_value, event }
    }

    fn __richcmp__(&self, py: Python<'_>, other: &Self, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => (self == other).into_py(py),
            CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented()
        }
    }
}

impl From<RustEvent> for Event {
    fn from(event: RustEvent) -> Self {
        let kv = event.kv().unwrap();
        let key = String::from_utf8(kv.key().to_owned()).unwrap();
        let value = String::from_utf8(kv.value().to_owned()).unwrap();
        let prev_value = None;
        let event = EventType(event.event_type());
        Self { key, value, prev_value, event }
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

    fn watch(&self, key: String) -> Watch {
        let client = self.0.clone();
        Watch::new(client, key, None)
    }

    fn watch_prefix(&self, key: String) -> Watch {
        let client = self.0.clone();
        let options = WatchOptions::new().with_prefix();
        Watch::new(client, key, Some(options))
    }
}

struct Stream {
    stream: RustStream,
    events: Vec<Event>,
    index: usize,
}

impl Stream {
    fn new(stream: RustStream) -> Self {
        Self {
            stream,
            events: Vec::new(),
            index: 0,
        }
    }

    async fn next(&mut self) -> Option<Result<Event, Error>> {
        if self.index < self.events.len() {
            let event = self.events[self.index].clone();
            self.index += 1;
            return Some(Ok(event));
        }
        let option = self.stream.next().await;
        let result = match option {
            Some(result) => result,
            None => return None
        };
        let response = match result {
            Ok(response) => response,
            Err(error) => return Some(Err(Error(error)))
        };
        let mut events = Vec::new();
        for event in response.events() {
            events.push(event.clone().into());
        }
        self.events = events;
        let event = self.events[0].clone();
        self.index = 1;
        Some(Ok(event))
    }
}

#[pyclass]
#[derive(Clone)]
struct Watch {
    client: Arc<Mutex<RustClient>>,
    key: String,
    options: Option<WatchOptions>,
    stream: Option<Arc<Mutex<Stream>>>,
}

impl Watch {
    fn new(client: Arc<Mutex<RustClient>>, key: String, options: Option<WatchOptions>) -> Self {
        Self {
            client,
            key,
            options,
            stream: None,
        }
    }

    async fn init(&mut self) -> Result<(), Error> {
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
                None => return Err(PyStopAsyncIteration::new_err(()))
            };
            Ok(result?)
        });
        Some(result.unwrap().into())
    }
}

#[pymodule]
#[pyo3(name = "etcd_client")]
fn init(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<Client>()?;
    module.add_class::<Event>()?;
    module.add_class::<EventType>()?;
    Ok(())
}

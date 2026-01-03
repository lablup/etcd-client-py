use etcd_client::Event as EtcdClientEvent;
use etcd_client::EventType as EtcdClientEventType;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::BoundObject;

// Note: Event = namedtuple("Event", "key event value"), not asyncio.Event, threading.Event
#[pyclass(get_all, name = "WatchEvent")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PyWatchEvent {
    key: Vec<u8>,
    value: Vec<u8>,
    event: PyWatchEventType,
    prev_value: Option<Vec<u8>>,
}

#[pymethods]
impl PyWatchEvent {
    #[new]
    #[pyo3(signature = (key, value, event, prev_value))]
    fn new(
        key: Vec<u8>,
        value: Vec<u8>,
        event: PyWatchEventType,
        prev_value: Option<Vec<u8>>,
    ) -> Self {
        Self {
            key,
            value,
            event,
            prev_value,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Event(event={:?}, key={:?}, value={:?}, prev_value={:?})",
            self.event, self.key, self.value, self.prev_value
        )
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        match op {
            CompareOp::Eq => (self == other)
                .into_pyobject(py)
                .map_err(Into::into)
                .map(BoundObject::into_any)
                .map(BoundObject::unbind),
            CompareOp::Ne => (self != other)
                .into_pyobject(py)
                .map_err(Into::into)
                .map(BoundObject::into_any)
                .map(BoundObject::unbind),
            _ => Ok(py.NotImplemented()),
        }
    }
}

impl From<EtcdClientEvent> for PyWatchEvent {
    fn from(event: EtcdClientEvent) -> Self {
        let kv = event.kv().unwrap();
        let key = kv.key().to_owned();
        let value = kv.value().to_owned();
        let prev_value = None;
        let event = PyWatchEventType(event.event_type());
        Self {
            key,
            value,
            event,
            prev_value,
        }
    }
}

#[pyclass(name = "WatchEventType")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PyWatchEventType(EtcdClientEventType);

#[pymethods]
impl PyWatchEventType {
    #[classattr]
    const PUT: Self = Self(EtcdClientEventType::Put);

    #[classattr]
    const DELETE: Self = Self(EtcdClientEventType::Delete);

    pub fn __repr__(&self) -> String {
        match self.0 {
            EtcdClientEventType::Put => "WatchEventType.PUT".to_string(),
            EtcdClientEventType::Delete => "WatchEventType.DELETE".to_string(),
        }
    }

    pub fn __richcmp__(
        &self,
        py: Python,
        rhs: &PyWatchEventType,
        op: CompareOp,
    ) -> PyResult<Py<PyAny>> {
        match op {
            CompareOp::Eq => (self.0 == rhs.0)
                .into_pyobject(py)
                .map_err(Into::into)
                .map(BoundObject::into_any)
                .map(BoundObject::unbind),
            CompareOp::Ne => (self.0 != rhs.0)
                .into_pyobject(py)
                .map_err(Into::into)
                .map(BoundObject::into_any)
                .map(BoundObject::unbind),
            _ => Ok(py.NotImplemented()),
        }
    }
}

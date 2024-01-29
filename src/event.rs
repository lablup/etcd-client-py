use etcd_client::Event as EtcdClientEvent;
use etcd_client::EventType as EtcdClientEventType;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

// Note: Event = namedtuple("Event", "key event value"), not asyncio.Event, threading.Event
#[pyclass(get_all, name = "Event")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PyEvent {
    key: String,
    value: String,
    event: PyEventType,
    prev_value: Option<String>,
}

#[pymethods]
impl PyEvent {
    #[new]
    #[pyo3(signature = (key, value, event, prev_value))]
    fn new(key: String, value: String, event: PyEventType, prev_value: Option<String>) -> Self {
        Self {
            key,
            value,
            event,
            prev_value,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Event(event={:?}, key={}, value={}, prev_value={:?})",
            self.event, self.key, self.value, self.prev_value
        )
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => (self == other).into_py(py),
            CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}

impl From<EtcdClientEvent> for PyEvent {
    fn from(event: EtcdClientEvent) -> Self {
        let kv = event.kv().unwrap();
        let key = String::from_utf8(kv.key().to_owned()).unwrap();
        let value = String::from_utf8(kv.value().to_owned()).unwrap();
        let prev_value = None;
        let event = PyEventType(event.event_type());
        Self {
            key,
            value,
            event,
            prev_value,
        }
    }
}

#[pyclass(name = "EventType")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PyEventType(EtcdClientEventType);

#[pymethods]
impl PyEventType {
    #[classattr]
    const PUT: Self = Self(EtcdClientEventType::Put);

    #[classattr]
    const DELETE: Self = Self(EtcdClientEventType::Delete);

    pub fn __repr__(&self) -> String {
        match self.0 {
            EtcdClientEventType::Put => "PUT".to_string(),
            EtcdClientEventType::Delete => "DELETE".to_string(),
        }
    }

    pub fn __richcmp__(&self, py: Python, rhs: &PyEventType, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => (self.0 == rhs.0).into_py(py),
            CompareOp::Ne => (self.0 != rhs.0).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}

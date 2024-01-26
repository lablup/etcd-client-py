use etcd_client::Event as RustEvent;
use etcd_client::EventType as RustEventType;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

#[pyclass]
#[derive(PartialEq, Eq, Clone)]
pub struct EventType(RustEventType);

#[pymethods]
impl EventType {
    #[classattr]
    const PUT: Self = Self(RustEventType::Put);

    #[classattr]
    const DELETE: Self = Self(RustEventType::Delete);
}

#[pyclass]
#[derive(PartialEq, Eq, Clone)]
pub struct Event {
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
        Self {
            key,
            value,
            prev_value,
            event,
        }
    }

    fn __richcmp__(&self, py: Python<'_>, other: &Self, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => (self == other).into_py(py),
            CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented(),
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
        Self {
            key,
            value,
            prev_value,
            event,
        }
    }
}

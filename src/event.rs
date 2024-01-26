use etcd_client::Event as EtcdClientEvent;
use etcd_client::EventType as EtcdClientEventType;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

#[pyclass]
#[derive(PartialEq, Eq, Clone)]
pub struct PyEventType(EtcdClientEventType);

#[pymethods]
impl PyEventType {
    #[classattr]
    const PUT: Self = Self(EtcdClientEventType::Put);

    #[classattr]
    const DELETE: Self = Self(EtcdClientEventType::Delete);
}

#[pyclass(name = "Event")]
#[derive(PartialEq, Eq, Clone)]
pub struct PyEvent {
    key: String,
    value: String,
    event: PyEventType,
    prev_value: Option<String>,
}

#[pymethods]
impl PyEvent {
    #[new]
    #[pyo3(signature = (key, value, prev_value, event))]
    fn new(key: String, value: String, prev_value: Option<String>, event: PyEventType) -> Self {
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
            prev_value,
            event,
        }
    }
}

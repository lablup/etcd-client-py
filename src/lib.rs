mod client;
mod communicator;
mod error;
mod event;
mod stream;
mod utils;
mod watch;

use client::PyClient;
use communicator::PyCommunicator;
use error::ClientError;
use event::{PyEvent, PyEventType};
use pyo3::prelude::*;
use watch::PyWatch;

#[pymodule]
fn etcd_client(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyClient>()?;
    module.add_class::<PyCommunicator>()?;
    module.add_class::<PyEvent>()?;
    module.add_class::<PyEventType>()?;
    module.add_class::<PyWatch>()?;

    module.add("ClientError", py.get_type::<ClientError>())?;
    Ok(())
}

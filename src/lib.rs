mod client;
mod communicator;
mod compare;
mod condvar;
mod error;
mod event;
mod request_generator;
mod stream;
mod transaction;
mod txn_response;
mod utils;
mod watch;

use client::PyClient;
use communicator::PyCommunicator;
use compare::{PyCompare, PyCompareOp};
use condvar::PyCondVar;
use error::ClientError;
use event::{PyWatchEvent, PyWatchEventType};
use pyo3::prelude::*;
use transaction::{PyTxn, PyTxnOp};
use txn_response::PyTxnResponse;
use watch::PyWatch;

#[pymodule]
fn etcd_client(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyClient>()?;
    module.add_class::<PyCommunicator>()?;

    module.add_class::<PyWatch>()?;
    module.add_class::<PyWatchEvent>()?;
    module.add_class::<PyWatchEventType>()?;

    module.add_class::<PyCondVar>()?;
    module.add_class::<PyCompare>()?;
    module.add_class::<PyCompareOp>()?;

    module.add_class::<PyTxn>()?;
    module.add_class::<PyTxnOp>()?;
    module.add_class::<PyTxnResponse>()?;

    module.add("ClientError", py.get_type::<ClientError>())?;
    Ok(())
}

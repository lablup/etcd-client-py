mod client;
mod communicator;
mod compare;
mod condvar;
mod error;
mod lock_manager;
mod txn;
mod txn_response;
mod watch;
mod watch_event;
mod watch_event_stream;

use client::{PyClient, PyConnectOptions};
use communicator::PyCommunicator;
use compare::{PyCompare, PyCompareOp};
use condvar::PyCondVar;
use error::{
    ClientError, ElectError, EndpointError, GRpcStatusError, InvalidArgsError,
    InvalidHeaderValueError, InvalidUriError, IoError, LeaseKeepAliveError, PyGRpcStatusCode,
    TransportError, Utf8Error, WatchError,
};
use lock_manager::PyEtcdLockOption;
use pyo3::prelude::*;
use txn::{PyTxn, PyTxnOp};
use txn_response::PyTxnResponse;
use watch::PyWatch;
use watch_event::{PyWatchEvent, PyWatchEventType};

#[pymodule]
fn etcd_client(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyClient>()?;
    module.add_class::<PyConnectOptions>()?;
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
    module.add_class::<PyEtcdLockOption>()?;

    module.add_class::<PyGRpcStatusCode>()?;

    module.add("ClientError", py.get_type::<ClientError>())?;
    module.add("GRpcStatusError", py.get_type::<GRpcStatusError>())?;
    module.add("InvalidArgsError", py.get_type::<InvalidArgsError>())?;
    module.add("IoError", py.get_type::<IoError>())?;
    module.add("InvalidUriError", py.get_type::<InvalidUriError>())?;
    module.add("TransportError", py.get_type::<TransportError>())?;
    module.add("WatchError", py.get_type::<WatchError>())?;
    module.add("Utf8Error", py.get_type::<Utf8Error>())?;
    module.add("LeaseKeepAliveError", py.get_type::<LeaseKeepAliveError>())?;
    module.add("ElectError", py.get_type::<ElectError>())?;
    module.add(
        "InvalidHeaderValueError",
        py.get_type::<InvalidHeaderValueError>(),
    )?;
    module.add("EndpointError", py.get_type::<EndpointError>())?;
    Ok(())
}

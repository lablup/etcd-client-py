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

use pyo3::prelude::*;

#[pymodule]
mod etcd_client {

    use pyo3::prelude::*;
    use crate::error::{
        ClientError, ElectError, EndpointError, GRPCStatusError, InvalidArgsError,
        InvalidHeaderValueError, InvalidUriError, IoError, LeaseKeepAliveError,
        TransportError, Utf8Error, WatchError,
    };

    #[pymodule_export]
    use crate::txn::{PyTxn, PyTxnOp};

    #[pymodule_export]
    use crate::txn_response::PyTxnResponse;

    #[pymodule_export]
    use crate::lock_manager::PyEtcdLockOption;

    #[pymodule_export]
    use crate::client::{PyClient, PyConnectOptions};

    #[pymodule_export]
    use crate::communicator::PyCommunicator;

    #[pymodule_export]
    use crate::compare::{PyCompare, PyCompareOp};

    #[pymodule_export]
    use crate::condvar::PyCondVar;

    #[pymodule_export]
    use crate::watch::PyWatch;

    #[pymodule_export]
    use crate::watch_event::{PyWatchEvent, PyWatchEventType};

    #[pymodule_export]
    use crate::error::PyGRPCStatusCode;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let py = m.py();
        m.add("ClientError", py.get_type::<ClientError>())?;
        m.add("GRPCStatusError", py.get_type::<GRPCStatusError>())?;
        m.add("InvalidArgsError", py.get_type::<InvalidArgsError>())?;
        m.add("IoError", py.get_type::<IoError>())?;
        m.add("InvalidUriError", py.get_type::<InvalidUriError>())?;
        m.add("TransportError", py.get_type::<TransportError>())?;
        m.add("WatchError", py.get_type::<WatchError>())?;
        m.add("Utf8Error", py.get_type::<Utf8Error>())?;
        m.add("LeaseKeepAliveError", py.get_type::<LeaseKeepAliveError>())?;
        m.add("ElectError", py.get_type::<ElectError>())?;
        m.add(
            "InvalidHeaderValueError",
            py.get_type::<InvalidHeaderValueError>(),
        )?;
        m.add("EndpointError", py.get_type::<EndpointError>())?;
        Ok(())
    }
}

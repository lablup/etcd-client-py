use pyo3::{create_exception, exceptions::PyException, types::PyDict, PyErr, Python, *};
use std::fmt::Debug;

create_exception!(etcd_client, ClientError, PyException);
create_exception!(etcd_client, GRpcStatusError, ClientError);
create_exception!(etcd_client, InvalidArgsError, ClientError);
create_exception!(etcd_client, InvalidUriError, ClientError);
create_exception!(etcd_client, IoError, ClientError);
create_exception!(etcd_client, TransportError, ClientError);
create_exception!(etcd_client, WatchError, ClientError);
create_exception!(etcd_client, Utf8Error, ClientError);
create_exception!(etcd_client, LeaseKeepAliveError, ClientError);
create_exception!(etcd_client, ElectError, ClientError);
create_exception!(etcd_client, InvalidHeaderValueError, ClientError);
create_exception!(etcd_client, EndpointError, ClientError);
create_exception!(etcd_client, LockError, ClientError);

#[pyclass(name = "GRpcStatusCode")]
pub enum PyGRpcStatusCode {
    Ok = 0,
    Cancelled = 1,
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    NotFound = 5,
    AlreadyExists = 6,
    PermissionDenied = 7,
    ResourceExhausted = 8,
    FailedPrecondition = 9,
    Aborted = 10,
    OutOfRange = 11,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
    DataLoss = 15,
    Unauthenticated = 16,
}

#[derive(Debug)]
#[pyclass(name = "ClientError")]
pub struct PyClientError(pub etcd_client::Error);

impl From<PyClientError> for PyErr {
    fn from(error: PyClientError) -> Self {
        match &error.0 {
            etcd_client::Error::GRpcStatus(e) => Python::with_gil(|py| {
                let error_details = PyDict::new(py);
                error_details.set_item("code", e.code() as u64).unwrap();
                error_details
                    .set_item("details", e.details().to_vec())
                    .unwrap();
                error_details
                    .set_item("message", e.message().to_owned())
                    .unwrap();

                let kv_args: PyObject = error_details.into_py(py);
                GRpcStatusError::new_err(kv_args)
            }),
            etcd_client::Error::InvalidArgs(e) => {
                InvalidArgsError::new_err(format!("InvalidArgsError(err={})", e))
            }
            etcd_client::Error::InvalidUri(e) => {
                InvalidUriError::new_err(format!("InvalidUriError(err={})", e))
            }
            etcd_client::Error::IoError(e) => IoError::new_err(format!("IoError(err={})", e)),
            etcd_client::Error::TransportError(e) => {
                TransportError::new_err(format!("TransportError(err={})", e))
            }
            etcd_client::Error::WatchError(e) => {
                WatchError::new_err(format!("WatchError(err={})", e))
            }
            etcd_client::Error::Utf8Error(e) => Utf8Error::new_err(format!("Utf8Error(err={})", e)),
            etcd_client::Error::LeaseKeepAliveError(e) => {
                LeaseKeepAliveError::new_err(format!("LeaseKeepAliveError(err={})", e))
            }
            etcd_client::Error::ElectError(e) => {
                ElectError::new_err(format!("ElectError(err={})", e))
            }
            etcd_client::Error::InvalidHeaderValue(e) => {
                InvalidHeaderValueError::new_err(format!("InvalidHeaderValueError(err={})", e))
            }
            etcd_client::Error::EndpointError(e) => {
                EndpointError::new_err(format!("EndpointError(err={})", e))
            }
            etcd_client::Error::LockError(e) => LockError::new_err(format!("LockError(err={})", e)),
        }
    }
}

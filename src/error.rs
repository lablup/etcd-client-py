use etcd_client::Error as RustError;
use pyo3::{create_exception, exceptions::PyException, PyErr};
use std::fmt::Debug;

create_exception!(etcd_client, ClientError, PyException);

#[derive(Debug)]
pub struct Error(pub RustError);

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        ClientError::new_err(error.0.to_string())
    }
}

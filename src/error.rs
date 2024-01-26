use pyo3::{create_exception, exceptions::PyException, PyErr};
use std::fmt::Debug;

create_exception!(etcd_client, ClientError, PyException);

#[derive(Debug)]
pub struct Error(pub etcd_client::Error);

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        ClientError::new_err(error.0.to_string())
    }
}

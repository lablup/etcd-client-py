use etcd_client::TxnResponse;
use pyo3::prelude::*;

#[derive(Clone)]
#[pyclass(name = "TxnResponse")]
pub struct PyTxnResponse(pub TxnResponse);

// TODO: Add ResponseHeader, TxnOpResponse
#[pymethods]
impl PyTxnResponse {
    pub fn succeeded(&self) -> PyResult<bool> {
        Ok(self.0.succeeded())
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

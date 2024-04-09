use etcd_client::{DeleteOptions, GetOptions, PutOptions, Txn, TxnOp};
use pyo3::{prelude::*, types::PyBytes};

use crate::compare::PyCompare;

#[derive(Debug, Clone)]
#[pyclass(name = "TxnOp")]
pub struct PyTxnOp(pub TxnOp);

#[pymethods]
impl PyTxnOp {
    #[staticmethod]
    fn get(key: &PyBytes) -> PyResult<Self> {
        let key = key.as_bytes().to_vec();
        let options = GetOptions::new();
        Ok(PyTxnOp(TxnOp::get(key, Some(options))))
    }

    #[staticmethod]
    fn put(key: &PyBytes, value: &PyBytes) -> PyResult<Self> {
        let key = key.as_bytes().to_vec();
        let value = value.as_bytes().to_vec();
        let options = PutOptions::new();
        Ok(PyTxnOp(TxnOp::put(key, value, Some(options))))
    }

    #[staticmethod]
    fn delete(key: &PyBytes) -> PyResult<Self> {
        let key = key.as_bytes().to_vec();
        let options = DeleteOptions::new();
        Ok(PyTxnOp(TxnOp::delete(key, Some(options))))
    }

    #[staticmethod]
    fn txn(txn: PyTxn) -> PyResult<Self> {
        Ok(PyTxnOp(TxnOp::txn(txn.0)))
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
#[pyclass(name = "Txn")]
pub struct PyTxn(pub Txn);

#[pymethods]
impl PyTxn {
    #[new]
    fn new() -> Self {
        PyTxn(Txn::new())
    }

    fn when(&self, compares: Vec<PyCompare>) -> PyResult<Self> {
        let compares = compares.into_iter().map(|c| c.0).collect::<Vec<_>>();
        Ok(PyTxn(self.0.clone().when(compares)))
    }

    fn and_then(&self, operations: Vec<PyTxnOp>) -> PyResult<Self> {
        let operations = operations.into_iter().map(|c| c.0).collect::<Vec<_>>();
        Ok(PyTxn(self.0.clone().and_then(operations)))
    }

    fn or_else(&self, operations: Vec<PyTxnOp>) -> PyResult<Self> {
        let operations = operations.into_iter().map(|c| c.0).collect::<Vec<_>>();
        Ok(PyTxn(self.0.clone().or_else(operations)))
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

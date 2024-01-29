use etcd_client::{DeleteOptions, GetOptions, PutOptions, Txn, TxnOp};
use pyo3::prelude::*;

use crate::compare::PyCompare;

#[derive(Clone)]
#[pyclass(name = "TxnOp")]
pub struct PyTxnOp(TxnOp);

#[pymethods]
impl PyTxnOp {
    #[staticmethod]
    fn get(key: String) -> PyResult<Self> {
        let options = GetOptions::new();
        Ok(PyTxnOp(TxnOp::get(key, Some(options))))
    }
    #[staticmethod]
    fn put(key: String, value: String) -> PyResult<Self> {
        let options = PutOptions::new();
        Ok(PyTxnOp(TxnOp::put(key, value, Some(options))))
    }

    #[staticmethod]
    fn delete(key: String) -> PyResult<Self> {
        let options = DeleteOptions::new();
        Ok(PyTxnOp(TxnOp::delete(key, Some(options))))
    }
}

#[derive(Clone)]
#[pyclass(name = "Txn")]
pub struct PyTxn(Txn);

#[pymethods]
impl PyTxn {
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
}

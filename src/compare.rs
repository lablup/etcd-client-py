use etcd_client::{Compare, CompareOp};
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp as PyO3CompareOp;

#[derive(Clone)]
#[pyclass(name = "CompareOp")]
pub struct PyCompareOp(CompareOp);

#[pymethods]
impl PyCompareOp {
    #[classattr]
    const EQUAL: Self = Self(CompareOp::Equal);
    #[classattr]
    const GREATER: Self = Self(CompareOp::Greater);
    #[classattr]
    const LESS: Self = Self(CompareOp::Less);
    #[classattr]
    const NOT_EQUAL: Self = Self(CompareOp::NotEqual);

    pub fn __repr__(&self) -> String {
        match self.0 {
            CompareOp::Equal => "EQUAL".to_owned(),
            CompareOp::Greater => "GREATER".to_owned(),
            CompareOp::Less => "LESS".to_owned(),
            CompareOp::NotEqual => "NOT_EQUAL".to_owned(),
        }
    }

    pub fn __richcmp__(&self, py: Python, rhs: &PyCompareOp, op: PyO3CompareOp) -> PyObject {
        match op {
            PyO3CompareOp::Eq => (self.0 == rhs.0).into_py(py),
            PyO3CompareOp::Ne => (self.0 != rhs.0).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}

#[derive(Clone)]
#[pyclass(name = "Compare")]
pub struct PyCompare(pub Compare);

#[pymethods]
impl PyCompare {
    #[staticmethod]
    fn version(key: String, cmp: PyCompareOp, version: i64) -> PyResult<Self> {
        Ok(PyCompare(Compare::version(key, cmp.0, version)))
    }

    #[staticmethod]
    fn create_revision(key: String, cmp: PyCompareOp, revision: i64) -> PyResult<Self> {
        Ok(PyCompare(Compare::create_revision(key, cmp.0, revision)))
    }

    #[staticmethod]
    fn mod_revision(key: String, cmp: PyCompareOp, revision: i64) -> PyResult<Self> {
        Ok(PyCompare(Compare::mod_revision(key, cmp.0, revision)))
    }

    #[staticmethod]
    fn value(key: String, cmp: PyCompareOp, value: String) -> PyResult<Self> {
        Ok(PyCompare(Compare::value(key, cmp.0, value)))
    }

    #[staticmethod]
    fn lease(key: String, cmp: PyCompareOp, lease: i64) -> PyResult<Self> {
        Ok(PyCompare(Compare::lease(key, cmp.0, lease)))
    }

    fn with_range(&self, end: String) -> PyResult<Self> {
        Ok(PyCompare(self.0.clone().with_range(end)))
    }

    fn with_prefix(&self) -> PyResult<Self> {
        Ok(PyCompare(self.0.clone().with_prefix()))
    }
}

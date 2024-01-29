use etcd_client::{Compare, CompareOp};
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp as PyO3CompareOp;

// #[derive(Clone)]
// #[pyclass(name = "CompareOp")]
// pub struct PyCompareOp(CompareOp);

// #[pymethods]
// impl PyCompareOp {
//     #[classattr]
//     const EQUAL: Self = Self(CompareOp::Equal);
//     #[classattr]
//     const GREATER: Self = Self(CompareOp::Greater);
//     #[classattr]
//     const LESS: Self = Self(CompareOp::Less);
//     #[classattr]
//     const NOT_EQUAL: Self = Self(CompareOp::NotEqual);

//     pub fn __repr__(&self) -> String {
//         match self.0 {
//             CompareOp::Equal => "EQUAL".to_owned(),
//             CompareOp::Greater => "GREATER".to_owned(),
//             CompareOp::Less => "LESS".to_owned(),
//             CompareOp::NotEqual => "NOT_EQUAL".to_owned(),
//         }
//     }

//     pub fn __richcmp__(&self, py: Python, rhs: &PyCompareOp, op: PyO3CompareOp) -> PyObject {
//         match op {
//             PyO3CompareOp::Eq => (self.0 == rhs.0).into_py(py),
//             PyO3CompareOp::Ne => (self.0 != rhs.0).into_py(py),
//             _ => py.NotImplemented(),
//         }
//     }
// }

#[derive(Clone)]
#[pyclass(name = "Compare")]
pub struct PyCompare(pub Compare);

#[pymethods]
impl PyCompare {
    // fn when(&self) {
    //     self.inner.
    // }

}

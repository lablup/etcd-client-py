use pyo3::{pyclass, *};
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

#[pyclass(name = "CondVar")]
#[derive(Clone)]
pub struct PyCondVar {
    inner: Arc<Notify>,
    condition: Arc<Mutex<bool>>,
}

#[pymethods]
impl PyCondVar {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(Notify::new()),
            condition: Arc::new(Mutex::new(false)),
        }
    }

    pub fn wait<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let inner = self.inner.clone();
        let condition = self.condition.clone();
        future_into_py(py, async move {
            while !*condition.lock().await {
                inner.notified().await;
            }
            Ok(())
        })
    }

    pub fn notify_waiters<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let inner = self.inner.clone();
        let condition = self.condition.clone();
        future_into_py(py, async move {
            *condition.lock().await = true;
            inner.notify_waiters();
            Ok(())
        })
    }
}

impl PyCondVar {
    pub async fn _notify_waiters(&self) {
        let inner = self.inner.clone();
        let condition = self.condition.clone();
        *condition.lock().await = true;
        inner.notify_waiters();
    }
}

use etcd_client::Client as RustClient;
use etcd_client::{DeleteOptions, GetOptions, WatchOptions};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::Error;
use crate::utils::nested_hashmap::{
    convert_pydict_to_nested_map, insert_into_map, put_recursive, NestedHashMap,
};
use crate::Watch;

#[pyclass]
pub struct Communicator(pub Arc<Mutex<RustClient>>);

#[pymethods]
impl Communicator {
    fn get<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.get(key, None).await;
            result
                .map(|response| {
                    let kvs = response.kvs();
                    if !kvs.is_empty() {
                        Some(String::from_utf8(kvs[0].value().to_owned()).unwrap())
                    } else {
                        None
                    }
                })
                .map_err(|e| Error(e).into())
        })
    }

    fn get_prefix<'a>(&'a self, py: Python<'a>, prefix: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = GetOptions::new().with_prefix();
            let response = client.get(prefix.clone(), Some(options)).await.unwrap();

            let mut result = NestedHashMap::new();
            for kv in response.kvs() {
                let key = String::from_utf8(kv.key().to_owned())
                    .unwrap()
                    .strip_prefix(&format!("{}/", prefix))
                    .unwrap()
                    .to_owned();

                let value = String::from_utf8(kv.value().to_owned()).unwrap();
                let parts: Vec<&str> = key.split('/').collect();
                insert_into_map(&mut result, &parts, value);
            }

            Ok(result)
        })
    }

    fn put<'a>(&'a self, py: Python<'a>, key: String, value: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.put(key, value, None).await;
            result.map(|_| ()).map_err(|e| Error(e).into())
        })
    }

    fn put_prefix<'a>(
        &'a self,
        py: Python<'a>,
        prefix: String,
        dict: &PyDict,
    ) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        let dict = convert_pydict_to_nested_map(py, dict).unwrap();
        future_into_py(py, async move {
            let result = put_recursive(client, prefix.as_str(), &dict).await;
            result
                .map(|_| ())
                // .map_err(|e| Error(e).into())
                .unwrap();
            Ok(())
        })
    }

    fn delete<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let result = client.delete(key, None).await;
            result.map(|_| ()).map_err(|e| Error(e).into())
        })
    }

    fn delete_prefix<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = DeleteOptions::new().with_prefix();
            let result = client.delete(key, Some(options)).await;
            result.map(|_| ()).map_err(|e| Error(e).into())
        })
    }

    // fn replace<'a>(&'a self, py: Python<'a>, key: String, initial_val: String, new_val: String) -> PyResult<&'a PyAny> {
    //     let client = self.0.clone();

    //     future_into_py(py, async move {
    //         let mut client = client.lock().await;

    //         // etcd에서 현재 값을 조회합니다.
    //         match client.get(key, None).await {
    //             Ok(response) => {
    //                 if let Some(key_value) = response.kvs().get(0) {
    //                     if key_value.value_str().unwrap().to_owned() == initial_val {
    //                         // 현재 값이 initial_val과 일치하면 new_val로 업데이트합니다.
    //                         match client.put(key, new_val, None).await {
    //                             Ok(_) => Ok(true.to_object(py)), // 성공적으로 변경됨
    //                             Err(e) => Err(PyErr::from(Error::from(e))),
    //                         }
    //                     } else {
    //                         Ok(false.to_object(py)) // 현재 값이 initial_val과 일치하지 않음
    //                     }
    //                 } else {
    //                     Ok(false.to_object(py)) // 키가 존재하지 않음
    //                 }
    //             },
    //             Err(e) => Err(PyErr::from(Error::from(e))),
    //         }
    //     })
    // }

    fn keys_prefix<'a>(&'a self, py: Python<'a>, key: String) -> PyResult<&'a PyAny> {
        let client = self.0.clone();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let options = GetOptions::new().with_prefix();
            let result = client.get(key, Some(options)).await;
            result
                .map(|response| {
                    let mut result = Vec::new();
                    let kvs = response.kvs();
                    for kv in kvs {
                        let key = String::from_utf8(kv.key().to_owned()).unwrap();
                        result.push(key);
                    }
                    result
                })
                .map_err(|e| Error(e).into())
        })
    }

    fn watch(&self, key: String) -> Watch {
        let client = self.0.clone();
        Watch::new(client, key, None)
    }

    fn watch_prefix(&self, key: String) -> Watch {
        let client = self.0.clone();
        let options = WatchOptions::new().with_prefix();
        Watch::new(client, key, Some(options))
    }
}
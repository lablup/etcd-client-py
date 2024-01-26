use async_recursion::async_recursion;
use etcd_client::{Client as EtcdClient, Error};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::utils::url::encode_string;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass(extends=PyDict)]
pub struct NestedHashMap(pub HashMap<String, NestedHashMapValue>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NestedHashMapValue {
    StringValue(String),
    MapValue(NestedHashMap),
}

impl IntoPy<Py<PyAny>> for NestedHashMap {
    fn into_py(self, py: Python) -> Py<PyAny> {
        let dict = PyDict::new(py);
        for (key, value) in self.0 {
            match value {
                NestedHashMapValue::StringValue(s) => {
                    dict.set_item(key, s).unwrap();
                }
                NestedHashMapValue::MapValue(inner_map) => {
                    let inner_dict: Py<PyAny> = inner_map.clone().into_py(py);
                    dict.set_item(key, inner_dict.as_ref(py)).unwrap();
                }
            }
        }
        dict.into()
    }
}

impl NestedHashMap {
    pub fn new() -> Self {
        NestedHashMap(HashMap::new())
    }
}

impl Deref for NestedHashMap {
    type Target = HashMap<String, NestedHashMapValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NestedHashMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn convert_pydict_to_nested_map(py: Python, py_dict: &PyDict) -> PyResult<NestedHashMap> {
    let mut map = NestedHashMap::new();
    for (key, value) in py_dict.iter() {
        let key = key.extract::<String>()?;

        if let Ok(inner_dict) = value.downcast::<PyDict>() {
            map.insert(
                key,
                NestedHashMapValue::MapValue(convert_pydict_to_nested_map(py, inner_dict)?),
            );
        } else if let Ok(val_str) = value.extract::<String>() {
            map.insert(key, NestedHashMapValue::StringValue(val_str));
        } else {
            unreachable!("Invalid type")
        }
    }
    Ok(map)
}

#[async_recursion]
pub async fn put_recursive(
    client: Arc<Mutex<EtcdClient>>,
    prefix: &str,
    dict: &HashMap<String, NestedHashMapValue>,
) -> Result<(), Error> {
    for (key, value) in dict {
        match value {
            NestedHashMapValue::StringValue(val_str) => {
                let mut client = client.lock().await;

                let full_key = if key.is_empty() {
                    prefix.to_owned()
                } else {
                    format!("{}/{}", prefix, encode_string(key))
                };

                client.put(full_key, val_str.clone(), None).await;
            }
            NestedHashMapValue::MapValue(map) => {
                put_recursive(
                    client.clone(),
                    &format!("{}/{}", prefix, encode_string(key)),
                    &map.0,
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub fn insert_into_map(map: &mut NestedHashMap, remaining_keys: &[&str], value: String) {
    if remaining_keys.is_empty() {
        return;
    }

    let first_key = remaining_keys[0].to_string();

    if remaining_keys.len() == 1 {
        map.insert(first_key, NestedHashMapValue::StringValue(value));
    } else {
        if let NestedHashMapValue::MapValue(inner_map) = map
            .entry(first_key.clone())
            .or_insert(NestedHashMapValue::MapValue(NestedHashMap::new()))
        {
            insert_into_map(inner_map, &remaining_keys[1..], value);
        } else {
            // TODO: Check if this is possible
            unreachable!();
        }
    }
}

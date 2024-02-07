use crate::{
    client::PyClient,
    communicator::PyCommunicator,
    error::{GRpcStatusError, LockError, PyClientError},
};
use etcd_client::{Client as EtcdClient, LockOptions};

use pyo3::prelude::*;
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[derive(Debug, Clone)]
#[pyclass(get_all, set_all, name = "EtcdLockOption")]
pub struct PyEtcdLockOption {
    pub lock_name: String,
    pub timeout: Option<f64>,
    pub ttl: Option<i64>,
}

#[pymethods]
impl PyEtcdLockOption {
    #[new]
    fn new(lock_name: String, timeout: Option<f64>, ttl: Option<i64>) -> Self {
        Self {
            lock_name,
            timeout,
            ttl,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "EtcdLockOption(lock_name={}, timeout={:?}, ttl={:?})",
            self.lock_name, self.timeout, self.ttl
        ))
    }
}

pub struct EtcdLockManager {
    pub client: PyClient,
    pub lock_name: String,
    pub ttl: Option<i64>,
    pub timeout_seconds: Option<f64>,
    pub lock_id: Option<String>,
    pub lease_id: Option<i64>,
    pub lease_keepalive_task: Option<tokio::task::JoinHandle<Result<(), PyClientError>>>,
}

impl EtcdLockManager {
    pub fn new(client: PyClient, lock_opt: PyEtcdLockOption) -> Self {
        Self {
            client,
            lock_name: lock_opt.lock_name,
            ttl: lock_opt.ttl,
            timeout_seconds: lock_opt.timeout,
            lock_id: None,
            lease_id: None,
            lease_keepalive_task: None,
        }
    }

    pub async fn handle_aenter(&mut self) -> PyResult<PyCommunicator> {
        let client = self.client.clone();
        let mut client = EtcdClient::connect(client.endpoints, Some(client.connect_options.0))
            .await
            .map_err(PyClientError)?;

        self.lease_id = match self.ttl {
            Some(ttl) => {
                let lease_grant_res = client.lease_grant(ttl, None).await.map_err(PyClientError)?;
                let lease_id = lease_grant_res.id();

                let mut client_to_move = client.clone();

                self.lease_keepalive_task = Some(tokio::spawn(async move {
                    let (mut lease_keeper, _lease_stream) = client_to_move
                        .lease_keep_alive(lease_id)
                        .await
                        .map_err(PyClientError)?;

                    loop {
                        sleep(Duration::from_secs((ttl / 10) as u64)).await;
                        lease_keeper.keep_alive().await.map_err(PyClientError)?;
                    }
                }));

                Some(lease_id)
            }
            None => None,
        };

        let timeout_result: Result<Result<(), PyClientError>, tokio::time::error::Elapsed> =
            timeout(
                Duration::from_secs_f64(self.timeout_seconds.unwrap()),
                async {
                    let lock_req_options = self
                        .lease_id
                        .map(|lease_id| LockOptions::new().with_lease(lease_id));

                    let lock_res = client
                        .lock(self.lock_name.clone().as_bytes(), lock_req_options)
                        .await
                        .map_err(PyClientError)?;

                    self.lock_id = Some(
                        std::str::from_utf8(lock_res.key())
                            .expect("Invalid utf8 chars includeded in lock request!")
                            .to_owned(),
                    );
                    Ok(())
                },
            )
            .await;

        match timeout_result {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(e.into()),
            Err(timedout_err) => {
                if let Some(lease_id) = self.lease_id {
                    match client.lease_revoke(lease_id).await {
                        Ok(_lease_revoke_res) => {}
                        Err(e) => match e {
                            etcd_client::Error::GRpcStatus(status)
                                if status.code() != tonic::Code::NotFound =>
                            {
                                return Err(GRpcStatusError::new_err(status.to_string()))
                            }
                            _ => return Err(PyClientError(e).into()),
                        },
                    }
                    return Err(LockError::new_err(timedout_err.to_string()));
                }
            }
        }

        if let Some(ref lease_keepalive_task) = self.lease_keepalive_task {
            lease_keepalive_task.abort();
        }

        Ok(PyCommunicator::new(client))
    }

    pub async fn handle_aexit(&mut self) -> PyResult<()> {
        let client = self.client.clone();
        let mut client = EtcdClient::connect(client.endpoints, Some(client.connect_options.0))
            .await
            .map_err(PyClientError)?;

        if self.lock_id.is_none() {
            return Err(LockError::new_err(
                "Attempting to release EtcdLock before it has been acquired!".to_string(),
            ));
        }

        if let Some(ref lease_keepalive_task) = self.lease_keepalive_task {
            lease_keepalive_task.abort();
        }

        if let Some(lease_id) = self.lease_id {
            client.lease_revoke(lease_id).await.map_err(PyClientError)?;
        } else {
            client
                .unlock(self.lock_id.as_ref().unwrap().as_bytes())
                .await
                .map_err(PyClientError)?;
        }

        self.lock_id = None;
        self.lease_id = None;

        Ok(())
    }
}

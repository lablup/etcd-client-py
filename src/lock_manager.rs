use crate::{
    client::PyClient,
    communicator::PyCommunicator,
    error::{GRPCStatusError, LockError, PyClientError},
};
use etcd_client::{Client as EtcdClient, LockOptions};

use pyo3::prelude::*;
use std::{future::ready, time::Duration};
use tokio::time::{sleep, timeout};

#[derive(Debug, Clone)]
#[pyclass(get_all, set_all, name = "EtcdLockOption")]
pub struct PyEtcdLockOption {
    pub lock_name: Vec<u8>,
    pub timeout: Option<f64>,
    pub ttl: Option<i64>,
}

#[pymethods]
impl PyEtcdLockOption {
    #[new]
    #[pyo3(signature = (lock_name, timeout=None, ttl=None))]
    fn new(lock_name: Vec<u8>, timeout: Option<f64>, ttl: Option<i64>) -> Self {
        Self {
            lock_name,
            timeout,
            ttl,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "EtcdLockOption(lock_name={:?}, timeout={:?}, ttl={:?})",
            self.lock_name, self.timeout, self.ttl
        ))
    }
}

pub struct EtcdLockManager {
    pub client: PyClient,
    pub lock_name: Vec<u8>,
    pub ttl: Option<i64>,
    pub timeout_seconds: Option<f64>,
    pub lock_id: Option<Vec<u8>>,
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

    async fn try_lock(&mut self, client: &mut EtcdClient) -> Result<(), PyClientError> {
        let lock_req_options = self
            .lease_id
            .map(|lease_id| LockOptions::new().with_lease(lease_id));

        let lock_res = client
            .lock(self.lock_name.clone(), lock_req_options)
            .await
            .map_err(PyClientError)?;

        self.lock_id = Some(lock_res.key().to_vec());
        Ok(())
    }

    pub async fn handle_aenter(&mut self) -> PyResult<PyCommunicator> {
        let client = self.client.clone();
        let mut client = EtcdClient::connect(client.endpoints, Some(client.connect_options.0))
            .await
            .map_err(PyClientError)?;

        let mut self_ = scopeguard::guard(self, |self_| {
            if let Some(ref lease_keepalive_task) = self_.lease_keepalive_task {
                lease_keepalive_task.abort();
            }
        });

        self_.lease_id = match self_.ttl {
            Some(ttl) => {
                let lease_grant_res = client.lease_grant(ttl, None).await.map_err(PyClientError)?;
                let lease_id = lease_grant_res.id();

                let mut client_to_move = client.clone();

                self_.lease_keepalive_task = Some(tokio::spawn(async move {
                    let (mut lease_keeper, _lease_stream) = client_to_move
                        .lease_keep_alive(lease_id)
                        .await
                        .map_err(PyClientError)?;

                    loop {
                        sleep(Duration::from_secs_f64((ttl as f64) / 10.0)).await;
                        lease_keeper.keep_alive().await.map_err(PyClientError)?;
                    }
                }));

                Some(lease_id)
            }
            None => None,
        };

        let timeout_result: Result<Result<(), PyClientError>, tokio::time::error::Elapsed> =
            match self_.timeout_seconds {
                Some(seconds) => {
                    timeout(
                        Duration::from_secs_f64(seconds),
                        self_.try_lock(&mut client),
                    )
                    .await
                }
                None => ready(Ok(self_.try_lock(&mut client).await)).await,
            };

        match timeout_result {
            Ok(Ok(_)) => Ok(PyCommunicator::new(client)),
            Ok(Err(try_lock_err)) => Err(try_lock_err.into()),
            Err(timedout_err) => {
                if let Some(lease_id) = self_.lease_id {
                    if let Err(etcd_client::Error::GRpcStatus(status)) =
                        client.lease_revoke(lease_id).await
                    {
                        if status.code() != tonic::Code::NotFound {
                            return Err(GRPCStatusError::new_err(status.to_string()));
                        }
                    }
                }
                Err(LockError::new_err(timedout_err.to_string()))
            }
        }
    }

    pub async fn handle_aexit(&mut self) -> PyResult<()> {
        let client = self.client.clone();
        let mut client = EtcdClient::connect(client.endpoints, Some(client.connect_options.0))
            .await
            .map_err(PyClientError)?;

        match self.lock_id {
            None => {
                return Err(LockError::new_err(
                    "Attempting to release EtcdLock before it has been acquired!".to_string(),
                ));
            }
            Some(ref lock_id) => {
                if let Some(ref lease_keepalive_task) = self.lease_keepalive_task {
                    lease_keepalive_task.abort();
                }

                if let Some(lease_id) = self.lease_id {
                    if let Err(etcd_client::Error::GRpcStatus(status)) =
                        client.lease_revoke(lease_id).await
                    {
                        if status.code() != tonic::Code::NotFound {
                            return Err(GRPCStatusError::new_err(status.to_string()));
                        }
                    }
                } else {
                    client
                        .unlock(lock_id.to_owned())
                        .await
                        .map_err(PyClientError)?;
                }
            }
        }

        self.lock_id = None;
        self.lease_id = None;

        Ok(())
    }
}

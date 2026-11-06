from collections.abc import Iterator

import pytest
import trustme
from etcd_client import Client, ConnectOptions, TlsOptions
from testcontainers.core.container import DockerContainer
from testcontainers.core.wait_strategies import LogMessageWaitStrategy

from .harness import AsyncEtcd, ConfigScopes, HostPortPair

ETCD_VER = "v3.5.14"
_CERTS_MOUNT_PATH = "/certs"

_etcd_command = """/usr/local/bin/etcd
  --name s1
  --data-dir /etcd-data
  --listen-client-urls http://0.0.0.0:2379
  --advertise-client-urls http://0.0.0.0:2379
  --listen-peer-urls http://0.0.0.0:2380
  --initial-advertise-peer-urls http://0.0.0.0:2380
  --initial-cluster s1=http://0.0.0.0:2380
  --initial-cluster-token tkn
  --initial-cluster-state new
  --log-level info
  --logger zap
  --log-outputs stderr
"""

_etcd_command_requiring_client_certificate = f"""/usr/local/bin/etcd
  --name s1
  --data-dir /etcd-data
  --listen-client-urls https://0.0.0.0:2379
  --advertise-client-urls https://127.0.0.1:2379
  --cert-file {_CERTS_MOUNT_PATH}/server.pem
  --key-file {_CERTS_MOUNT_PATH}/server-key.pem
  --trusted-ca-file {_CERTS_MOUNT_PATH}/ca.pem
  --client-cert-auth
  --log-level info
  --logger zap
  --log-outputs stderr
"""


@pytest.fixture(scope="session")
def etcd_container():
    container = (
        DockerContainer(f"gcr.io/etcd-development/etcd:{ETCD_VER}", command=_etcd_command)
        .with_exposed_ports(2379)
        .waiting_for(LogMessageWaitStrategy("ready to serve client requests"))
    )
    with container:
        yield container


@pytest.fixture(scope="session")
def client_with_ca_certificate_and_identity(
    tmp_path_factory: pytest.TempPathFactory,
) -> Iterator[Client]:
    ca = trustme.CA()
    server_cert = ca.issue_cert("127.0.0.1")
    client_cert = ca.issue_cert("etcd-client-py-test")

    certs_dir = tmp_path_factory.mktemp("etcd-certs")
    ca.cert_pem.write_to_path(certs_dir / "ca.pem")
    server_cert.cert_chain_pems[0].write_to_path(certs_dir / "server.pem")
    server_cert.private_key_pem.write_to_path(certs_dir / "server-key.pem")

    container = (
        DockerContainer(
            f"gcr.io/etcd-development/etcd:{ETCD_VER}",
            command=_etcd_command_requiring_client_certificate,
        )
        .with_exposed_ports(2379)
        .with_volume_mapping(str(certs_dir), _CERTS_MOUNT_PATH, "ro")
        .waiting_for(LogMessageWaitStrategy("ready to serve client requests"))
    )
    tls = (
        TlsOptions()
        .with_ca_certificate(ca.cert_pem.bytes())
        .with_identity(client_cert.cert_chain_pems[0].bytes(), client_cert.private_key_pem.bytes())
    )
    with container:
        yield Client(
            [f"https://127.0.0.1:{container.get_exposed_port(2379)}"],
            connect_options=ConnectOptions().with_tls(tls),
        )


@pytest.fixture
async def etcd(etcd_container):
    etcd_port = etcd_container.get_exposed_port(2379)
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port=etcd_port),
        namespace="test",
        scope_prefix_map={
            ConfigScopes.GLOBAL: "global",
            ConfigScopes.SGROUP: "sgroup/testing",
            ConfigScopes.NODE: "node/i-test",
        },
    )
    async with etcd:
        try:
            await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
            await etcd.delete_prefix("", scope=ConfigScopes.SGROUP)
            await etcd.delete_prefix("", scope=ConfigScopes.NODE)
            yield etcd
        finally:
            await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
            await etcd.delete_prefix("", scope=ConfigScopes.SGROUP)
            await etcd.delete_prefix("", scope=ConfigScopes.NODE)


@pytest.fixture
async def gateway_etcd(etcd_container):
    etcd_port = etcd_container.get_exposed_port(2379)
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port=etcd_port),
        namespace="test",
        scope_prefix_map={
            ConfigScopes.GLOBAL: "",
        },
    )
    async with etcd:
        try:
            await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
            yield etcd
        finally:
            await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)

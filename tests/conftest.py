import pytest
from testcontainers.core.container import DockerContainer
from testcontainers.core.waiting_utils import wait_for_logs

from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair

ETCD_VER = "v3.5.14"

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


@pytest.fixture(scope="session")
def etcd_container():
    with DockerContainer(
        f"gcr.io/etcd-development/etcd:{ETCD_VER}",
        command=_etcd_command,
    ).with_bind_ports("2379/tcp", 2379) as container:
        wait_for_logs(container, "ready to serve client requests")
        yield


@pytest.fixture
async def etcd(etcd_container):
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port=2379),
        namespace="test",
        scope_prefix_map={
            ConfigScopes.GLOBAL: "global",
            ConfigScopes.SGROUP: "sgroup/testing",
            ConfigScopes.NODE: "node/i-test",
        },
    )
    try:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        await etcd.delete_prefix("", scope=ConfigScopes.SGROUP)
        await etcd.delete_prefix("", scope=ConfigScopes.NODE)
        yield etcd
    finally:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        await etcd.delete_prefix("", scope=ConfigScopes.SGROUP)
        await etcd.delete_prefix("", scope=ConfigScopes.NODE)
        await etcd.close()
        del etcd


@pytest.fixture
async def gateway_etcd(etcd_container):
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port=2379),
        namespace="test",
        scope_prefix_map={
            ConfigScopes.GLOBAL: "",
        },
    )
    try:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        yield etcd
    finally:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        del etcd

import pytest
from testcontainers.core.container import DockerContainer
from testcontainers.core.wait_strategies import LogMessageWaitStrategy

from .harness import AsyncEtcd, ConfigScopes, HostPortPair

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
    container = (
        DockerContainer(f"gcr.io/etcd-development/etcd:{ETCD_VER}", command=_etcd_command)
        .with_exposed_ports(2379)
        .waiting_for(LogMessageWaitStrategy("ready to serve client requests"))
    )
    with container:
        yield container


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

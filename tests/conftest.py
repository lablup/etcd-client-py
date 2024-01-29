import pytest
from tests.harness import AsyncEtcd, ConfigScopes, HostPortPair


@pytest.fixture
async def etcd():
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
        return etcd
    finally:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        await etcd.delete_prefix("", scope=ConfigScopes.SGROUP)
        await etcd.delete_prefix("", scope=ConfigScopes.NODE)
        await etcd.close()
        del etcd


@pytest.fixture
async def gateway_etcd():
    etcd = AsyncEtcd(
        addr=HostPortPair(host="127.0.0.1", port=2379),
        namespace="test",
        scope_prefix_map={
            ConfigScopes.GLOBAL: "",
        },
    )
    try:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        return etcd
    finally:
        await etcd.delete_prefix("", scope=ConfigScopes.GLOBAL)
        del etcd

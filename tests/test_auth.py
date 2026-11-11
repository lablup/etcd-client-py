import asyncio
from collections.abc import Iterator

import pytest
from etcd_client import Client, ConnectOptions, GRPCStatusError
from testcontainers.core.container import DockerContainer
from testcontainers.core.wait_strategies import LogMessageWaitStrategy

from .conftest import ETCD_VER

_ROOT_PASSWORD = "root-password"
_AUTH_TOKEN_TTL_SECONDS = 1
_WAIT_FOR_TOKEN_EXPIRY_SECONDS = 3.0

_etcd_command_with_short_token_ttl = f"""/usr/local/bin/etcd
  --name s1
  --data-dir /etcd-data
  --listen-client-urls http://0.0.0.0:2379
  --advertise-client-urls http://0.0.0.0:2379
  --auth-token simple
  --auth-token-ttl {_AUTH_TOKEN_TTL_SECONDS}
  --log-level info
  --logger zap
  --log-outputs stderr
"""


@pytest.fixture(scope="module")
def auth_enabled_etcd_endpoint() -> Iterator[str]:
    container = (
        DockerContainer(
            f"gcr.io/etcd-development/etcd:{ETCD_VER}",
            command=_etcd_command_with_short_token_ttl,
        )
        .with_exposed_ports(2379)
        .waiting_for(LogMessageWaitStrategy("ready to serve client requests"))
    )
    with container:
        container.exec(["etcdctl", "user", "add", "root", "--new-user-password", _ROOT_PASSWORD])
        container.exec(["etcdctl", "auth", "enable"])
        yield f"http://127.0.0.1:{container.get_exposed_port(2379)}"


@pytest.mark.asyncio
async def test_request_after_token_expiry_succeeds_with_auto_token_refresh(
    auth_enabled_etcd_endpoint: str,
) -> None:
    client = Client(
        [auth_enabled_etcd_endpoint],
        connect_options=ConnectOptions()
        .with_user("root", _ROOT_PASSWORD)
        .with_auto_token_refresh(True),
    )
    async with client.connect() as communicator:
        await communicator.put(b"auth-key", b"auth-value")
        await asyncio.sleep(_WAIT_FOR_TOKEN_EXPIRY_SECONDS)

        assert bytes(await communicator.get(b"auth-key")) == b"auth-value"


@pytest.mark.asyncio
async def test_request_after_token_expiry_fails_without_auto_token_refresh(
    auth_enabled_etcd_endpoint: str,
) -> None:
    client = Client(
        [auth_enabled_etcd_endpoint],
        connect_options=ConnectOptions().with_user("root", _ROOT_PASSWORD),
    )
    async with client.connect() as communicator:
        await communicator.put(b"auth-key", b"auth-value")
        await asyncio.sleep(_WAIT_FOR_TOKEN_EXPIRY_SECONDS)

        with pytest.raises(GRPCStatusError, match="invalid auth token"):
            await communicator.get(b"auth-key")

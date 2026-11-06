import asyncio
import socket
from collections.abc import Iterator

import pytest
from etcd_client import Client, ConnectOptions, GRPCStatusError, TlsOptions

_ETCD_PASSWORD = "etcd-password"
_CLIENT_KEY_PEM = b"client-key-pem"
_HANDSHAKE_TIMEOUT_SECONDS = 0.2
_REQUEST_WAIT_LIMIT_SECONDS = 5.0


@pytest.fixture
def client_with_password_and_private_key() -> Client:
    connect_options = (
        ConnectOptions()
        .with_user("etcd-user", _ETCD_PASSWORD)
        .with_tls(TlsOptions().with_identity(b"client-cert-pem", _CLIENT_KEY_PEM))
    )
    return Client(["https://127.0.0.1:2379"], connect_options=connect_options)


@pytest.fixture
def client_of_server_silent_during_handshake() -> Iterator[Client]:
    with socket.create_server(("127.0.0.1", 0)) as silent_server:
        yield Client(
            [f"https://127.0.0.1:{silent_server.getsockname()[1]}"],
            connect_options=ConnectOptions().with_tls(
                TlsOptions().with_handshake_timeout(_HANDSHAKE_TIMEOUT_SECONDS)
            ),
        )


@pytest.mark.asyncio
async def test_reads_and_writes_with_ca_certificate_and_identity(
    client_with_ca_certificate_and_identity: Client,
) -> None:
    async with client_with_ca_certificate_and_identity.connect() as communicator:
        await communicator.put(b"tls-key", b"tls-value")
        assert bytes(await communicator.get(b"tls-key")) == b"tls-value"


@pytest.mark.asyncio
async def test_handshake_timeout_ends_request_to_silent_server(
    client_of_server_silent_during_handshake: Client,
) -> None:
    async def request_silent_server() -> None:
        async with client_of_server_silent_during_handshake.connect() as communicator:
            await communicator.get(b"tls-key")

    with pytest.raises(GRPCStatusError, match="TLS handshake timeout"):
        await asyncio.wait_for(request_silent_server(), _REQUEST_WAIT_LIMIT_SECONDS)


def test_client_repr_hides_password_and_private_key(
    client_with_password_and_private_key: Client,
) -> None:
    client_repr = repr(client_with_password_and_private_key)

    assert _ETCD_PASSWORD not in client_repr
    assert str(list(_CLIENT_KEY_PEM)) not in client_repr

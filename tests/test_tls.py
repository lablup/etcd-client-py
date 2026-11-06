import pytest
from etcd_client import Client, ConnectOptions, TlsOptions

_ETCD_PASSWORD = "etcd-password"
_CLIENT_KEY_PEM = b"client-key-pem"


@pytest.fixture
def client_with_password_and_private_key() -> Client:
    connect_options = (
        ConnectOptions()
        .with_user("etcd-user", _ETCD_PASSWORD)
        .with_tls(TlsOptions().with_identity(b"client-cert-pem", _CLIENT_KEY_PEM))
    )
    return Client(["https://127.0.0.1:2379"], connect_options=connect_options)


@pytest.mark.asyncio
async def test_reads_and_writes_with_ca_certificate_and_identity(
    client_with_ca_certificate_and_identity: Client,
) -> None:
    async with client_with_ca_certificate_and_identity.connect() as communicator:
        await communicator.put(b"tls-key", b"tls-value")
        assert bytes(await communicator.get(b"tls-key")) == b"tls-value"


def test_client_repr_hides_password_and_private_key(
    client_with_password_and_private_key: Client,
) -> None:
    client_repr = repr(client_with_password_and_private_key)

    assert _ETCD_PASSWORD not in client_repr
    assert str(list(_CLIENT_KEY_PEM)) not in client_repr

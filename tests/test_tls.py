from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import pytest
import trustme
from etcd_client import Client, ConnectOptions, TlsOptions
from testcontainers.core.container import DockerContainer
from testcontainers.core.wait_strategies import LogMessageWaitStrategy

from .conftest import ETCD_VER

_CERTS_MOUNT_PATH = "/certs"

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


@dataclass(frozen=True)
class _EtcdRequiringClientCertificate:
    endpoint: str
    ca_pem: bytes
    client_cert_pem: bytes
    client_key_pem: bytes


@pytest.fixture(scope="module")
def etcd_requiring_client_certificate(
    tmp_path_factory: pytest.TempPathFactory,
) -> Iterator[_EtcdRequiringClientCertificate]:
    ca = trustme.CA()
    server_cert = ca.issue_cert("127.0.0.1")
    client_cert = ca.issue_cert("etcd-client-py-test")

    certs_dir: Path = tmp_path_factory.mktemp("etcd-certs")
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
    with container:
        yield _EtcdRequiringClientCertificate(
            endpoint=f"https://127.0.0.1:{container.get_exposed_port(2379)}",
            ca_pem=ca.cert_pem.bytes(),
            client_cert_pem=client_cert.cert_chain_pems[0].bytes(),
            client_key_pem=client_cert.private_key_pem.bytes(),
        )


async def test_connects_with_ca_certificate_and_client_identity(
    etcd_requiring_client_certificate: _EtcdRequiringClientCertificate,
) -> None:
    etcd = etcd_requiring_client_certificate
    tls = (
        TlsOptions()
        .with_ca_certificate(etcd.ca_pem)
        .with_identity(etcd.client_cert_pem, etcd.client_key_pem)
    )
    client = Client([etcd.endpoint], connect_options=ConnectOptions().with_tls(tls))

    async with client.connect() as communicator:
        await communicator.put(b"tls-key", b"tls-value")
        assert bytes(await communicator.get(b"tls-key")) == b"tls-value"


def test_client_repr_hides_password_and_private_key() -> None:
    connect_options = (
        ConnectOptions()
        .with_user("etcd-user", "etcd-password")
        .with_tls(TlsOptions().with_identity(b"client-cert-pem", b"client-key-pem"))
    )

    client_repr = repr(Client(["https://127.0.0.1:2379"], connect_options=connect_options))

    assert "etcd-password" not in client_repr
    assert str(list(b"client-key-pem")) not in client_repr

"""
Type hints for Native Rust Extension
"""

from enum import Enum
from typing import Any, AsyncIterator, Final, Optional

class CompareOp:
    """ """

    EQUAL: Final[Any]
    """
    """
    NOT_EQUAL: Final[Any]
    """
    """
    GREATER: Final[Any]
    """
    """
    LESS: Final[Any]
    """
    """

class Compare:
    @staticmethod
    def version(key: str, cmp: "CompareOp", version: int) -> "Compare": ...
    @staticmethod
    def create_revision(key: str, cmp: "CompareOp", revision: int) -> "Compare": ...
    @staticmethod
    def mod_revision(key: str, cmp: "CompareOp", revision: int) -> "Compare": ...
    @staticmethod
    def value(key: str, cmp: "CompareOp", value: str) -> "Compare": ...
    @staticmethod
    def lease(key: str, cmp: "CompareOp", lease: int) -> "Compare": ...
    def with_range(self, end: list[int]) -> "Compare": ...
    def with_prefix(self) -> "Compare": ...

class Txn:
    def __init__(self) -> None: ...
    def when(self, compares: list["Compare"]) -> "Txn": ...
    def and_then(self, operations: list["TxnOp"]) -> "Txn": ...
    def or_else(self, operations: list["TxnOp"]) -> "Txn": ...

class TxnOp:
    @staticmethod
    def get(key: str) -> "TxnOp": ...
    @staticmethod
    def put(key: str, value: str) -> "TxnOp": ...
    @staticmethod
    def delete(key: str) -> "TxnOp": ...
    @staticmethod
    def txn(txn: "Txn") -> "TxnOp": ...

class TxnResponse:
    def succeeded(self) -> bool: ...
    def op_responses(self) -> None: ...

class Client:
    """ """

    def __init__(
        self, endpoints: list[str], options: Optional["ConnectOptions"] = None
    ) -> None:
        """ """
    def connect(self, options: Optional["ConnectOptions"] = None) -> "Client":
        """ """
    async def __aenter__(self) -> "Communicator":
        """ """
    async def __aexit__(self, *args) -> None:
        """ """

class ConnectOptions:
    def __init__(self) -> None: ...
    def with_user(self, user: str, password: str) -> "ConnectOptions": ...
    def with_keep_alive(self, interval: float, timeout: float) -> "ConnectOptions": ...
    def with_keep_alive_while_idle(self, enabled: bool) -> "ConnectOptions": ...
    def with_connect_timeout(self, connect_timeout: float) -> "ConnectOptions": ...
    def with_timeout(self, timeout: float) -> "ConnectOptions": ...
    def with_tcp_keepalive(self, tcp_keepalive: float) -> "ConnectOptions": ...

class Watch:
    """ """

    def __aiter__(self) -> AsyncIterator["WatchEvent"]:
        """ """
    async def __anext__(self) -> "WatchEvent":
        """ """

class CondVar:
    """ """

    def __init__(self) -> None:
        """ """
    async def wait(self) -> None:
        """ """
    async def notify_waiters(self) -> None:
        """ """

class Communicator:
    async def get(self, key: str) -> str:
        """ """
    async def get_prefix(self, key: str) -> dict[str, Any]:
        """ """
    async def put(self, key: str, value: str) -> None:
        """ """
    async def put_prefix(self, key: str, value: dict[str, Any]) -> None:
        """ """
    async def txn(self, txn: "Txn") -> "TxnResponse":
        """ """
    async def delete(self, key: str) -> None:
        """ """
    async def delete_prefix(self, key: str) -> None:
        """ """
    async def keys_prefix(self, key: str) -> list[str]:
        """ """
    async def replace(self, key: str, initial_value: str, new_value: str) -> bool:
        """ """
    async def lock(self, name: str) -> None:
        """ """
    async def unlock(self, key: str) -> None:
        """ """
    async def lease_grant(self, ttl: int) -> None:
        """ """
    async def lease_revoke(self, id: int) -> None:
        """ """
    async def lease_time_to_live(self, id: int) -> None:
        """ """
    def watch(
        self,
        key: str,
        *,
        once: Optional[bool] = False,
        ready_event: Optional["CondVar"] = None,
    ) -> "Watch":
        """ """
    def watch_prefix(
        self,
        key: str,
        *,
        once: Optional[bool] = False,
        ready_event: Optional["CondVar"] = None,
    ) -> "Watch":
        """ """

class WatchEvent:
    """ """

    key: str
    value: str
    event: "WatchEventType"
    prev_value: Optional[str]

    def __init__(
        key: str,
        value: str,
        event: "WatchEventType",
        prev_value: Optional[str] = None,
    ) -> None: ...

class WatchEventType:
    """ """

    PUT: Final[Any]
    """
    """
    DELETE: Final[Any]
    """
    """

class ClientError(Exception):
    """ """

class GRpcStatusError(ClientError):
    """ """

class InvalidArgsError(ClientError):
    """ """

class IoError(ClientError):
    """ """

class InvalidUriError(ClientError):
    """ """

class TransportError(ClientError):
    """ """

class WatchError(ClientError):
    """ """

class Utf8Error(ClientError):
    """ """

class LeaseKeepAliveError(ClientError):
    """ """

class ElectError(ClientError):
    """ """

class InvalidHeaderValueError(ClientError):
    """ """

class EndpointError(ClientError):
    """ """

class GRpcStatusCode(Enum):
    Ok = 0
    """The operation completed successfully."""

    Cancelled = 1
    """The operation was cancelled."""

    Unknown = 2
    """Unknown error."""

    InvalidArgument = 3
    """Client specified an invalid argument."""

    DeadlineExceeded = 4
    """Deadline expired before operation could complete."""

    NotFound = 5
    """Some requested entity was not found."""

    AlreadyExists = 6
    """Some entity that we attempted to create already exists."""

    PermissionDenied = 7
    """The caller does not have permission to execute the specified operation."""

    ResourceExhausted = 8
    """Some resource has been exhausted."""

    FailedPrecondition = 9
    """The system is not in a state required for the operation's execution."""

    Aborted = 10
    """The operation was aborted."""

    OutOfRange = 11
    """Operation was attempted past the valid range."""

    Unimplemented = 12
    """Operation is not implemented or not supported."""

    Internal = 13
    """Internal error."""

    Unavailable = 14
    """The service is currently unavailable."""

    DataLoss = 15
    """Unrecoverable data loss or corruption."""

    Unauthenticated = 16
    """The request does not have valid authentication credentials."""

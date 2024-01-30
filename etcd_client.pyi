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
    @classmethod
    def version(key: str, cmp: "CompareOp", version: int) -> "Compare": ...
    @classmethod
    def create_revision(key: str, cmp: "CompareOp", revision: int) -> "Compare": ...
    @classmethod
    def mod_revision(key: str, cmp: "CompareOp", revision: int) -> "Compare": ...
    @classmethod
    def value(key: str, cmp: "CompareOp", value: str) -> "Compare": ...
    @classmethod
    def lease(key: str, cmp: "CompareOp", lease: int) -> "Compare": ...
    def with_range(self, end: list[int]) -> "Compare": ...
    def with_prefix(self) -> "Compare": ...

class Txn:
    def __init__(self) -> None: ...
    def when(self, compares: list["Compare"]) -> "Txn": ...
    def and_then(self, operations: list["TxnOp"]) -> "Txn": ...
    def or_else(self, operations: list["TxnOp"]) -> "Txn": ...

class TxnOp:
    @classmethod
    def get(key: str) -> "TxnOp": ...
    @classmethod
    def put(key: str, value: str) -> "TxnOp": ...
    @classmethod
    def delete(key: str) -> "TxnOp": ...
    @classmethod
    def txn(txn: "Txn") -> "TxnOp": ...

class TxnResponse:
    def succeeded(self) -> bool: ...

class Client:
    """ """

    def __init__(
        self, endpoints: list[str], options: Optional["ConnectOptions"] = None
    ) -> None:
        """ """
    def connect(self) -> "Client":
        """ """
    async def __aenter__(self) -> "Communicator":
        """ """

class ConnectOptions:
    def __init__(self) -> None: ...
    def with_username(self, user: str, password: str) -> "ConnectOptions": ...
    def with_keep_alive(self, interval: int, timeout: int) -> "ConnectOptions": ...
    def with_keep_alive_while_idle(self, enabled: bool) -> "ConnectOptions": ...
    def with_connect_timeout(self, connect_timeout: int) -> "ConnectOptions": ...
    def with_timeout(self, timeout: int) -> "ConnectOptions": ...
    def with_tcp_keepalive(self, tcp_keepalive: int) -> "ConnectOptions": ...

class Watch:
    """ """

    async def __aiter__(self) -> AsyncIterator["Watch"]:
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
    event_type: "WatchEventType"
    prev_value: Optional[str]

    def __init__(
        key: str,
        value: str,
        event_type: "WatchEventType",
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

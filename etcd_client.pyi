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
    """
    Compares the version of the given key.
    """
    @classmethod
    def create_revision(key: str, cmp: "CompareOp", revision: int) -> "Compare": ...
    """
    Compares the creation revision of the given key.
    """
    @classmethod
    def mod_revision(key: str, cmp: "CompareOp", revision: int) -> "Compare": ...
    """
    Compares the last modified revision of the given key.
    """
    @classmethod
    def value(key: str, cmp: "CompareOp", value: str) -> "Compare": ...
    """
    Compares the value of the given key.
    """
    @classmethod
    def lease(key: str, cmp: "CompareOp", lease: int) -> "Compare": ...
    """
    Compares the lease id of the given key.
    """
    def with_range(self, end: list[int]) -> "Compare": ...
    """
    Sets the comparison to scan the range [key, end).
    """
    def with_prefix(self) -> "Compare": ...
    """
    Sets the comparison to scan all keys prefixed by the key.
    """

class Txn:
    """
    Transaction of multiple operations.
    """

    def __init__(self) -> None: ...
    """
    Creates a new transaction.
    """
    def when(self, compares: list["Compare"]) -> "Txn": ...
    """
    Takes a list of comparison. If all comparisons passed in succeed,
    the operations passed into `and_then()` will be executed. Or the operations
    passed into `or_else()` will be executed.
    """
    def and_then(self, operations: list["TxnOp"]) -> "Txn": ...
    """
    Takes a list of operations. The operations list will be executed, if the
    comparisons passed in `when()` succeed.
    """
    def or_else(self, operations: list["TxnOp"]) -> "Txn": ...
    """
    Takes a list of operations. The operations list will be executed, if the
    comparisons passed in `when()` fail. 
    """

class TxnOp:
    """
    Transaction operation.
    """

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
    """
    Returns `true` if the compare evaluated to true or `false` otherwise.
    """

class Client:
    """ """

    def __init__(
        self, endpoints: list[str], options: Optional["ConnectOptions"] = None
    ) -> None:
        """ """
    def connect(self, options: Optional["ConnectOptions"] = None) -> "Client":
        """ """
    async def __aenter__(self) -> "Communicator":
        """
        Connect to `etcd` servers from given `endpoints`.
        """

class ConnectOptions:
    def __init__(self) -> None: ...
    def with_user(self, user: str, password: str) -> "ConnectOptions": ...
    """
    name is the identifier for the distributed shared lock to be acquired.
    """
    def with_keep_alive(self, interval: int, timeout: int) -> "ConnectOptions": ...
    """
    Enable HTTP2 keep-alive with `interval` and `timeout`.
    """
    def with_keep_alive_while_idle(self, enabled: bool) -> "ConnectOptions": ...
    """
    Whether send keep alive pings even there are no active requests.
    If disabled, keep-alive pings are only sent while there are opened request/response streams.
    If enabled, pings are also sent when no streams are active.
    NOTE: Some implementations of gRPC server may send GOAWAY if there are too many pings.
          This would be useful if you meet some error like `too many pings`.
    """
    def with_connect_timeout(self, connect_timeout: int) -> "ConnectOptions": ...
    """Apply a timeout to connecting to the endpoint."""
    def with_timeout(self, timeout: int) -> "ConnectOptions": ...
    """Apply a timeout to each request."""
    def with_tcp_keepalive(self, tcp_keepalive: int) -> "ConnectOptions": ...
    """Enable TCP keepalive."""

class Communicator:
    async def get(self, key: str) -> str:
        """
        Gets the key from the key-value store.
        """
    async def get_prefix(self, key: str) -> dict[str, Any]:
        """
        Gets the key from the key-value store.
        """
    async def put(self, key: str, value: str) -> None:
        """
        Put the given key into the key-value store.
        A put request increments the revision of the key-value store
        and generates one event in the event history.
        """
    async def put_prefix(self, key: str, value: dict[str, Any]) -> None:
        """
        Put the given key into the key-value store.
        A put request increments the revision of the key-value store
        and generates one event in the event history.
        """
    async def txn(self, txn: "Txn") -> "TxnResponse":
        """
        Processes multiple operations in a single transaction.
        A txn request increments the revision of the key-value store
        and generates events with the same revision for every completed operation.
        It is not allowed to modify the same key several times within one txn.
        """
    async def delete(self, key: str) -> None:
        """
        Deletes the given key from the key-value store.
        """
    async def delete_prefix(self, key: str) -> None:
        """
        Deletes the given key from the key-value store.
        """
    async def keys_prefix(self, key: str) -> list[str]:
        """ """
    async def replace(self, key: str, initial_value: str, new_value: str) -> bool:
        """ """
    async def lock(self, name: str) -> None:
        """
        Lock acquires a distributed shared lock on a given named lock.
        On success, it will return a unique key that exists so long as the
        lock is held by the caller. This key can be used in conjunction with
        transactions to safely ensure updates to etcd only occur while holding
        lock ownership. The lock is held until Unlock is called on the key or the
        lease associate with the owner expires.
        """
    async def unlock(self, key: str) -> None:
        """
        Unlock takes a key returned by Lock and releases the hold on lock. The
        next Lock caller waiting for the lock will then be woken up and given
        ownership of the lock.
        """
    async def lease_grant(self, ttl: int) -> None:
        """
        Creates a lease which expires if the server does not receive a keepAlive
        within a given time to live period. All keys attached to the lease will be expired and
        deleted if the lease expires. Each expired key generates a delete event in the event history.
        """
    async def lease_revoke(self, id: int) -> None:
        """Revokes a lease. All keys attached to the lease will expire and be deleted."""
    async def lease_time_to_live(self, id: int) -> None:
        """Retrieves lease information."""
    def watch(
        self,
        key: str,
        *,
        once: Optional[bool] = False,
        ready_event: Optional["CondVar"] = None,
    ) -> "Watch":
        """
        Watches for events happening or that have happened. Both input and output
        are streams; the input stream is for creating and canceling watcher and the output
        stream sends events. The entire event history can be watched starting from the
        last compaction revision.
        """
    def watch_prefix(
        self,
        key: str,
        *,
        once: Optional[bool] = False,
        ready_event: Optional["CondVar"] = None,
    ) -> "Watch":
        """
        Watches for events happening or that have happened. Both input and output
        are streams; the input stream is for creating and canceling watcher and the output
        stream sends events. The entire event history can be watched starting from the
        last compaction revision.
        """

class Watch:
    """ """

    async def __aiter__(self) -> AsyncIterator["Watch"]:
        """ """
    async def __anext__(self) -> "WatchEvent":
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

class CondVar:
    """ """

    def __init__(self) -> None:
        """ """
    async def wait(self) -> None:
        """ """
    async def notify_waiters(self) -> None:
        """ """

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

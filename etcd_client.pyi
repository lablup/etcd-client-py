"""
Type hints for Native Rust Extension
"""

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
    def with_range(end: list[int]) -> "Compare": ...
    def with_prefix() -> "Compare": ...

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

    def __init__(self, endpoints: list[str]) -> None:
        """ """
    def connect(self) -> "Client":
        """ """
    async def __aenter__(self) -> "Communicator":
        """ """

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
    async def notify_all(self) -> None:
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
        key: str, value: str, event_type: "WatchEventType", prev_value: Optional[str]
    ) -> None: ...

class WatchEventType:
    """ """

    PUT: Final[Any]
    """
    """
    DELETE: Final[Any]
    """
    """

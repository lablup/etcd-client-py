"""
Type hints for Native Rust Extension
"""

from typing import Any, AsyncIterator, Final, Iterator, Optional

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
    async def __anext__(self) -> "Event":
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
    async def delete(self, key: str) -> None:
        """ """
    async def delete_prefix(self, key: str) -> None:
        """ """
    async def keys_prefix(self, key: str) -> list[str]:
        """ """
    async def replace(self, key: str, initial_value: str, new_value: str) -> bool:
        """ """
    def watch(
        self, key: str, *, ready_event: Optional["CondVar"] = None
    ) -> "Watch":
        """ """
    def watch_prefix(
        self, key: str, *, ready_event: Optional["CondVar"] = None
    ) -> "Watch":
        """ """

class Event:
    """ """
    key: str
    value: str
    event_type: "EventType"
    prev_value: Optional[str]

    def __init__(
        key: str, value: str, event_type: "EventType", prev_value: Optional[str]
    ) -> None: ...

class EventType:
    """ """

    PUT: Final[Any]
    """
    """
    DELETE: Final[Any]
    """
    """

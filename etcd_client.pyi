"""
Type hints for Native Rust Extension
"""

import asyncio
from typing import Any, Final, Optional

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

class Communicator:
    def get(self, key: str) -> str:
        """ """
    def get_prefix(self, key: str) -> dict:
        """ """
    def put(self, key: str, value: str) -> None:
        """ """
    def put_prefix(self, key: str, value: dict) -> None:
        """ """
    def delete(self, key: str) -> None:
        """ """
    def delete_prefix(self, key: str) -> None:
        """ """
    def keys_prefix(self, key: str) -> list[str]:
        """ """
    def replace(self, key: str, initial_value: str, new_value: str) -> bool:
        """ """
    def watch(
        self, key: str, *, once: Optional[bool], ready_event: Optional[asyncio.Event]
    ) -> "Watch":
        """ """
    def watch_prefix(
        self, key: str, *, once: Optional[bool], ready_event: Optional[asyncio.Event]
    ) -> "Watch":
        """ """

class Event:
    """ """

    def __init__(
        key: str, value: str, event: "EventType", prev_value: Optional[str]
    ) -> None: ...

class EventType:
    """ """

    PUT: Final[Any]
    """
    """
    DELETE: Final[Any]
    """
    """

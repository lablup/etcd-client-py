"""
Type hints for Native Rust Extension
"""

from typing import Any, Final, Optional

class Client:
    """ """

    def __init__(self, endpoints: list[str]) -> None:
        """ """
    def connect(self) -> "Client":
        """ """
    def __aenter__(self) -> "Communicator":
        """ """

class Watch:
    """ """

class Communicator:
    def get(self, key: str) -> str:
        """ """
    def get_prefix(self, key: str) -> dict:
        """ """
    def keys_prefix(self, key: str) -> list[str]:
        """ """
    def delete(self, key: str) -> None:
        """ """
    def delete_prefix(self, key: str) -> None:
        """ """
    def put(self, key: str, value: str) -> None:
        """ """
    def put_prefix(self, key: str, value: dict) -> None:
        """ """
    def replace(self, key: str, initial_value: str, new_value: str) -> bool:
        """ """
    def watch(self, key: str) -> "Watch":
        """ """
    def watch_prefix(self, key: str) -> "Watch":
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

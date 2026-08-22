"""What the algebra says should happen to one construct."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class Verdict(Enum):
    """The three cases that exhaust the space of foreign consumers."""

    OK = "ok"
    HOIST = "hoist"
    MOVE = "move"


@dataclass(frozen=True)
class Proposal:
    construct: str
    verdict: Verdict
    home: str
    target: str | None
    consumers: frozenset[str]

"""The outcome of iterating the taxonomy to a stable room assignment."""

from __future__ import annotations

from dataclasses import dataclass

from room_algebra.features.analysis.classes.proposal import Proposal


@dataclass(frozen=True)
class FixpointResult:
    rounds: int
    converged: bool
    history: list[tuple[int, int]]
    final: dict[str, Proposal]
    rooms: dict[str, str]

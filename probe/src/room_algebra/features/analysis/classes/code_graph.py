"""The construct-level dependency graph a room analysis runs over."""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass, field

from room_algebra.features.analysis.classes.construct import Construct


@dataclass
class CodeGraph:
    """Constructs, the edges between them, and the room each is assigned to.

    Edges are construct -> construct rather than file -> file. That is what
    makes the fixpoint precise: reassigning one declaration to another room
    rewrites only the edges that declaration owns, so the room graph can be
    recomputed exactly instead of approximated.
    """

    constructs: dict[str, Construct] = field(default_factory=dict)
    edges: dict[str, set[str]] = field(default_factory=lambda: defaultdict(set))
    room: dict[str, str] = field(default_factory=dict)
    _inbound: dict[str, set[str]] = field(default_factory=lambda: defaultdict(set))

    def add(self, construct: Construct, room: str) -> None:
        self.constructs[construct.qualname] = construct
        self.room[construct.qualname] = room

    def depend(self, source: str, target: str) -> None:
        if source != target:
            self.edges[source].add(target)
            self._inbound[target].add(source)

    def consumer_rooms(self, target: str) -> set[str]:
        """Every room holding at least one construct that depends on `target`."""
        return {self.room[src] for src in self._inbound.get(target, ()) if src in self.room}

    def room_edges(self) -> dict[str, set[str]]:
        out: dict[str, set[str]] = defaultdict(set)
        for src, targets in self.edges.items():
            home = self.room.get(src)
            if home is None:
                continue
            for tgt in targets:
                there = self.room.get(tgt)
                if there is not None and there != home:
                    out[home].add(there)
        return out

    def sinks(self) -> set[str]:
        """Rooms that supply without consuming — the only legal home for shared code."""
        edges = self.room_edges()
        return {r for r in set(self.room.values()) if not edges.get(r)}

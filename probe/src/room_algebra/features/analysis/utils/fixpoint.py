"""Iterate the taxonomy until the room assignment stops moving."""

from __future__ import annotations

from room_algebra.features.analysis.classes.code_graph import CodeGraph
from room_algebra.features.analysis.classes.fixpoint_result import FixpointResult
from room_algebra.features.analysis.classes.proposal import Verdict
from room_algebra.features.analysis.utils.classify import classify


def fixpoint(graph: CodeGraph, max_rounds: int = 12) -> FixpointResult:
    """Apply every proposal, recompute, repeat.

    Only `graph.room` is mutated, so the caller can restore the original
    assignment by keeping a copy of that one dict.

    Convergence is not guaranteed: MOVE can pull a construct out of a shared
    room that a later round wants to push back. Repeated states are detected
    and reported rather than silently iterated to the cap.
    """
    history: list[tuple[int, int]] = []
    seen: set[frozenset[tuple[str, str]]] = set()

    for completed in range(max_rounds):
        proposals = classify(graph)
        hoists = sum(1 for p in proposals.values() if p.verdict is Verdict.HOIST)
        moves = sum(1 for p in proposals.values() if p.verdict is Verdict.MOVE)
        history.append((hoists, moves))

        if hoists + moves == 0:
            return FixpointResult(completed, True, history, proposals, dict(graph.room))

        state = frozenset(graph.room.items())
        if state in seen:
            return FixpointResult(completed, False, history, proposals, dict(graph.room))
        seen.add(state)

        for proposal in proposals.values():
            if proposal.target is not None:
                graph.room[proposal.construct] = proposal.target

    return FixpointResult(max_rounds, False, history, classify(graph), dict(graph.room))

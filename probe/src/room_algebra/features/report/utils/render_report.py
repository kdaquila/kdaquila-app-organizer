"""Render one corpus's analysis as text."""

from __future__ import annotations

from collections import Counter

from room_algebra.features.analysis.classes.code_graph import CodeGraph
from room_algebra.features.analysis.classes.fixpoint_result import FixpointResult
from room_algebra.features.analysis.classes.module_facts import MODULE_SCOPE
from room_algebra.features.analysis.classes.proposal import Proposal, Verdict


def render_report(
    label: str,
    graph: CodeGraph,
    proposals: dict[str, Proposal],
    result: FixpointResult,
    show: int = 8,
) -> str:
    named = {k: v for k, v in proposals.items() if not k.endswith(f":{MODULE_SCOPE}")}
    counts = Counter(p.verdict for p in named.values())
    total = len(named)
    shared = sum(1 for p in named.values() if len(p.consumers) >= 2)
    cross = sum(1 for p in named.values() if p.consumers)
    edges = graph.room_edges()
    rooms = set(graph.room.values())
    sinks = graph.sinks()

    lines = [
        f"=== {label} ===",
        f"constructs {total}   rooms {len(rooms)}   sinks {len(sinks)}",
        # `import x.y` pseudo-constructs are excluded: they inflate every count
        # without naming anything a human could move.
        f"cross-room constructs {cross} ({_pct(cross, total)} — the rest never"
        f" leave their room)   shared by 2+ rooms: {shared}",
        "",
        f"round 1:  HOIST {counts[Verdict.HOIST]}"
        f" ({_pct(counts[Verdict.HOIST], total)} of all constructs)"
        f"   MOVE {counts[Verdict.MOVE]}   OK {counts[Verdict.OK]}",
        # HOIST + MOVE covers every cross-room construct outside a sink by
        # construction, so that ratio is arithmetic, not a result. HOIST as a
        # share of the whole codebase is the number worth reading.
        "",
        f"top {show} hoists by consumer-room count:",
    ]

    top = sorted(
        (p for p in named.values() if p.verdict is Verdict.HOIST),
        key=lambda p: (-len(p.consumers), p.construct),
    )[:show]
    lines += [f"  {len(p.consumers):3d}  {p.construct}" for p in top] or ["  (none)"]

    lines += [
        "",
        f"busiest rooms by out-degree: "
        + ", ".join(f"{r}({len(t)})" for r, t in sorted(edges.items(), key=lambda kv: -len(kv[1]))[:6]),
        "",
        f"fixpoint: {'converged' if result.converged else 'DID NOT converge'}"
        f" after {result.rounds} rounds",
        "  per-round (hoist, move): "
        + " -> ".join(f"({h},{m})" for h, m in result.history),
        f"  rooms before {len(rooms)}   after {len(set(result.rooms.values()))}",
        "",
    ]
    return "\n".join(lines)


def _pct(part: int, whole: int) -> str:
    return f"{100.0 * part / whole:.1f}%" if whole else "n/a"

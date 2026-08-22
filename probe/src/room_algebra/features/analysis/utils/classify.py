"""Apply the three-case taxonomy to every construct in a graph."""

from __future__ import annotations

from room_algebra.features.analysis.classes.code_graph import CodeGraph
from room_algebra.features.analysis.classes.proposal import Proposal, Verdict

SHARED_SUFFIX = ".__shared__"


def classify(graph: CodeGraph) -> dict[str, Proposal]:
    """One round of the taxonomy.

    Three cases exhaust the space of a construct's *foreign* consumer rooms:
    two or more means it is shared and belongs in a sink; exactly one means it
    is simply misfiled; none means it is already where it belongs.

    A construct already sitting in a sink is left alone when shared — that is
    what a sink is for — but is still pulled out when only one room wants it,
    which stops the shared room becoming a dumping ground.
    """
    sinks = graph.sinks()
    proposals: dict[str, Proposal] = {}

    for qualname, home in graph.room.items():
        foreign = graph.consumer_rooms(qualname) - {home}
        if len(foreign) >= 2 and home not in sinks:
            verdict, target = Verdict.HOIST, _shared_room(home)
        elif len(foreign) == 1:
            only = next(iter(foreign))
            verdict, target = (Verdict.MOVE, only) if only != home else (Verdict.OK, None)
        else:
            verdict, target = Verdict.OK, None
        proposals[qualname] = Proposal(
            construct=qualname,
            verdict=verdict,
            home=home,
            target=target,
            consumers=frozenset(foreign),
        )

    return proposals


def _shared_room(home: str) -> str:
    """Hoist to a sink derived from the current room, never to one global bucket.

    A single project-wide dumping ground would erase the locality the fixpoint
    is trying to discover; adjacent shared rooms can still merge later.
    """
    return home if home.endswith(SHARED_SUFFIX) else home + SHARED_SUFFIX

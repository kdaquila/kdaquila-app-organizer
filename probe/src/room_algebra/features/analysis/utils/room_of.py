"""Map a dotted module onto the room that currently holds it."""

from __future__ import annotations


def room_of(module: str, depth: int) -> str:
    """Truncate a module path to `depth` folders below the package root.

    Rooms are deliberately coarser than modules: the whole premise is that the
    file is furniture and the folder is the wall.
    """
    parts = module.split(".")
    return ".".join(parts[: 1 + depth])

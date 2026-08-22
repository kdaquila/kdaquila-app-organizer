"""What a single parsed module declares, imports, and references."""

from __future__ import annotations

from dataclasses import dataclass

# alias -> (relative level, source module or None, original name; "*" means the module itself)
ImportSpec = tuple[int, str | None, str]

MODULE_SCOPE = "<module>"
"""Pseudo-construct standing for a module's top-level code."""


@dataclass(frozen=True)
class ModuleFacts:
    module: str
    is_package: bool
    defines: dict[str, str]
    imports: dict[str, ImportSpec]
    uses: dict[str, set[str]]

"""Walk a package and build its construct-level dependency graph."""

from __future__ import annotations

from pathlib import Path

from room_algebra.features.analysis.classes.code_graph import CodeGraph
from room_algebra.features.analysis.classes.construct import Construct
from room_algebra.features.analysis.classes.module_facts import MODULE_SCOPE, ModuleFacts
from room_algebra.features.analysis.utils.collect_module import collect_module
from room_algebra.features.analysis.utils.room_of import room_of


def build_graph(root: Path, depth: int, skip_tests: bool = False) -> CodeGraph:
    """Index every module under `root`, then resolve every reference to a declaration.

    Re-export facades are followed to the true declaration site, so a name
    re-exported through `__init__.py` is attributed to the room that actually
    declares it rather than the room a consumer imported it through.
    """
    facts = _collect_all(root, skip_tests)
    graph = CodeGraph()

    for module, fact in facts.items():
        graph.add(Construct(module, MODULE_SCOPE, "module"), room_of(module, depth))
        for name, kind in fact.defines.items():
            graph.add(Construct(module, name, kind), room_of(module, depth))

    for module, fact in facts.items():
        for owner, aliases in fact.uses.items():
            source = f"{module}:{owner}"
            if source not in graph.constructs:
                continue
            for alias in aliases:
                target = _resolve(facts, module, alias)
                if target is not None and target in graph.constructs:
                    graph.depend(source, target)

    return graph


def _collect_all(root: Path, skip_tests: bool) -> dict[str, ModuleFacts]:
    facts: dict[str, ModuleFacts] = {}
    for path in sorted(root.rglob("*.py")):
        parts = list(path.relative_to(root.parent).parts)
        if skip_tests and any(p in {"test", "tests", "_test"} for p in parts[:-1]):
            continue
        is_package = parts[-1] == "__init__.py"
        parts = parts[:-1] if is_package else [*parts[:-1], parts[-1][:-3]]
        if not parts:
            continue
        module = ".".join(parts)
        fact = collect_module(path, module, is_package)
        if fact is not None:
            facts[module] = fact
    return facts


def _resolve(facts: dict[str, ModuleFacts], module: str, alias: str) -> str | None:
    level, source, original = facts[module].imports[alias]
    absolute = _absolute(module, facts[module].is_package, level, source)
    if absolute is None:
        return None
    if original == "*":
        return f"{absolute}:{MODULE_SCOPE}" if absolute in facts else None
    return _trace(facts, absolute, original, set())


def _trace(
    facts: dict[str, ModuleFacts], module: str, name: str, seen: set[tuple[str, str]]
) -> str | None:
    """Follow a name through re-export facades to the module that declares it."""
    if (module, name) in seen or module not in facts:
        return None
    seen.add((module, name))
    fact = facts[module]
    if name in fact.defines:
        return f"{module}:{name}"
    if f"{module}.{name}" in facts:
        return f"{module}.{name}:{MODULE_SCOPE}"
    if name in fact.imports:
        level, source, original = fact.imports[name]
        absolute = _absolute(module, fact.is_package, level, source)
        if absolute is None:
            return None
        if original == "*":
            return f"{absolute}:{MODULE_SCOPE}" if absolute in facts else None
        return _trace(facts, absolute, original, seen)
    return None


def _absolute(current: str, is_package: bool, level: int, source: str | None) -> str | None:
    if level == 0:
        return source
    parts = current.split(".")
    base = parts if is_package else parts[:-1]
    if level > 1:
        cut = level - 1
        base = base[:-cut] if cut < len(base) else []
    if not base:
        return None
    return ".".join([*base, source] if source else base)

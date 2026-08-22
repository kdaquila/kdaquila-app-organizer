"""Parse one module into the declarations, imports, and references it holds."""

from __future__ import annotations

import ast
from pathlib import Path

from room_algebra.features.analysis.classes.module_facts import (
    MODULE_SCOPE,
    ImportSpec,
    ModuleFacts,
)


def collect_module(path: Path, module: str, is_package: bool) -> ModuleFacts | None:
    """Return the facts for one file, or None if it will not parse.

    References are attributed per top-level construct, not per file: a class
    that never touches an import does not inherit its neighbour's dependencies.
    """
    try:
        tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
    except (SyntaxError, ValueError):
        return None

    defines: dict[str, str] = {}
    imports: dict[str, ImportSpec] = {}
    for node in tree.body:
        _record_declaration(node, defines)
        _record_import(node, imports)

    uses: dict[str, set[str]] = {}
    imported = set(imports)
    module_scope: set[str] = set()
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            uses[node.name] = _referenced_names(node) & imported
        elif not isinstance(node, (ast.Import, ast.ImportFrom)):
            module_scope |= _referenced_names(node) & imported
    uses[MODULE_SCOPE] = module_scope

    return ModuleFacts(
        module=module,
        is_package=is_package,
        defines=defines,
        imports=imports,
        uses=uses,
    )


def _record_declaration(node: ast.stmt, defines: dict[str, str]) -> None:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
        defines[node.name] = "def"
    elif isinstance(node, ast.ClassDef):
        defines[node.name] = "class"
    elif isinstance(node, ast.Assign):
        for target in node.targets:
            if isinstance(target, ast.Name):
                defines[target.id] = "binding"
    elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
        defines[node.target.id] = "binding"


def _record_import(node: ast.stmt, imports: dict[str, ImportSpec]) -> None:
    if isinstance(node, ast.ImportFrom):
        for alias in node.names:
            if alias.name != "*":
                imports[alias.asname or alias.name] = (node.level, node.module, alias.name)
    elif isinstance(node, ast.Import):
        for alias in node.names:
            # `import a.b.c` binds `a` unless aliased; only the aliased form
            # gives us a usable handle on the deep module.
            if alias.asname:
                imports[alias.asname] = (0, alias.name, "*")
            else:
                imports[alias.name.split(".")[0]] = (0, alias.name.split(".")[0], "*")


def _referenced_names(node: ast.AST) -> set[str]:
    return {child.id for child in ast.walk(node) if isinstance(child, ast.Name)}

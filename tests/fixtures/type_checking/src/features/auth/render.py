from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

try:
    import tomllib
except ImportError:
    tomllib = None


def render(items: "Sequence[str]") -> str:
    return ", ".join(items)

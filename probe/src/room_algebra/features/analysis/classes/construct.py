"""One top-level declaration, addressed by the module that declares it."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Construct:
    """A single top-level `def`/`class`/binding.

    `module` is the module that *declares* it, after re-exports have been
    followed — never the facade a consumer happened to import it through.
    """

    module: str
    name: str
    kind: str

    @property
    def qualname(self) -> str:
        return f"{self.module}:{self.name}"

    def __str__(self) -> str:
        return self.qualname

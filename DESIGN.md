# app-organizer — design

An opinionated, multi-language validator for **app folder conventions, file naming
conventions, and file content conventions**.

Zero config is the selling point: like Prettier, it should "just work" with sensible
locked-in defaults. An optional config file exists for the few things that genuinely
vary between projects.

Scope note: this tool validates **shape and naming**. It deliberately has **no opinions
about dependencies** — import direction is a separate concern with its own tools
(e.g. `import-linter`). Coupling dependency rules to the folder vocabulary would make
the default vocabulary load-bearing, which projects should be free to override.

## Implementation

- **Rust** CLI, distributed as a single static binary.
- **One crate**, with both `src/lib.rs` and `src/main.rs`. That yields a library for the
  future pyo3/napi wrappers on day one without workspace ceremony; splitting into
  `crates/core` + `crates/cli` later is mechanical, so paying for it now is premature.
- Crate name **`kdaquila-app-organizer`** (publishable, avoids squatting a generic name for
  a tool with one likely user); binary name **`app-organizer`** via `[[bin]]`, so the
  command stays typable and diagnostics read cleanly. No short alias — a second binary is
  more to explain than it saves.
- **tree-sitter** for the content layer (grammars exist for Python, Rust, C++, TypeScript).
- Wrapper packages come later and just shell out to the binary: `pip` (maturin),
  `npm` (napi-rs or per-platform optional deps), `cargo` (free).

Why Rust over Go: tree-sitter is a C library, so Go needs cgo — which destroys the
easy cross-compilation and static linking that are Go's main draw here. In Rust the
grammars build through `cc` as part of `cargo build`.

**Language support order:** Python first, then TypeScript (React UIs), then Rust and
C++ (Python extensions).

## Layer 1 — folder grammar

A path pattern is a sequence of **positional segments**, each with its own constraints.
Segments are positional (`folder1`, `folder2`) rather than semantic (`slice`, `area`)
so that the engine carries no architectural vocabulary and each language profile can
supply its own.

Legal shapes are an explicit **list of pattern variants**. A path must match exactly one
of them:

```
{root}/{folder1}/{folder2}/{folder3}/{kind}/{files}
{root}/{folder1}/{folder2}/{kind}/{files}
{root}/{folder1}/{kind}/{files}
```

### Segment rules

| segment | rule |
|---|---|
| `root` | closed set, default `["src", "tests"]` — the main thing users override |
| `folder1` | closed set, default `["app", "features", "pages", "shared"]` |
| `folder2`, `folder3` | freely named, `snake_case`, **must not** collide with a kind name |
| `kind` | closed set, mandatory, always a **leaf** (files only, never subfolders) |
| `files` | the terminator — where files are allowed |

### Depth is data, not code

The engine parses each pattern string into a segment list and looks up constraints by
segment name. Nothing about "three folder levels" is hard-coded — a project that wants a
fourth level adds a `{folder4}` variant plus a `[python.folder4]` block, and a project
that wants only two deletes a line.

Spelling the variants out as a list, rather than marking segments `optional = true`,
is deliberate:

- **No fill-order rule is needed.** With optional segments, `src/features/auth/functions/x.py`
  raises the question of whether `auth` bound to `folder2` or `folder3` — which matters as
  soon as their constraints differ. The shallow variant literally names `{folder2}`, so
  the question never arises.
- **`oneOf` becomes literal** rather than derived.
- Adding or removing a depth is one line either way.

### Rejected: alternation syntax

A single pattern with inline alternation
(`"{root}/{files} | ({folder1}/{kind}/{files} | ...)"`). That is a mini-regex language,
requiring grouping, precedence, and error messages that explain *which branch* failed.
Three plain strings need none of that, and they are greppable. The repetition is the
explicitness.

### Rejected: JSON-Schema-style nested config

Explored at length, because it is more precise than the flat list and genuinely
*expresses* rules the flat version has to check separately. Two encodings were sketched:

- **Nested `oneOf` tree** — each node declares a segment plus `children`, with `oneOf`
  reserved for structural branching and `enum` for allowed values (JSON Schema's own
  split).
- **Directory-tree-as-document** — feed the whole repo in as nested objects with `null`
  file leaves, and validate with a stock JSON Schema validator. This one works: because
  `oneOf` applies to a directory *object as a whole*, the branch is chosen once for all
  its children, so **per-directory homogeneity becomes structural** rather than a
  separate check. Depth limiting falls out as `$def` chaining
  (`dirBody2` → `dirBody3` → `kindsOnly`). Closed levels want
  `properties` + `additionalProperties: false`; free-named levels want `propertyNames`
  + `additionalProperties`. Every dir-level `$def` needs an explicit `"type": "object"`,
  or a *file* named `src` passes validation. `minProperties: 1` is load-bearing, since
  `{}` matches both `oneOf` branches and fails the exactly-one requirement with a
  baffling message.

**Why rejected: diagnostics.** JSON Schema validators report `oneOf` failures as "does
not match exactly one subschema" plus a JSON Pointer. This tool's entire output *is* its
error messages, so that is the wrong thing to trade away. Producing decent messages would
mean walking the tree by hand anyway — leaving the validator doing the easy half while we
write the hard half. The flat list gives "matched none of these 3 patterns:" followed by
the patterns themselves, which is the diagnostic we actually want.

Also: JSON has no comments, which is a poor fit for a file whose job is encoding
conventions. TOML suits the flat form well.

Possible future revival: publish the tree schema as a **machine-readable artifact**
describing the conventions — consumable by editors and other tools — while the Rust
engine keeps its own walk for error quality.

### Cross-cutting rules

- **No mixed segment types within a directory.** A given directory's children must all
  be `{folder}`s, or all `{kind}`s — never both. This is *per-directory*, not per-root,
  so `features/auth/` can be deep while `features/ping/` stays shallow. Sibling subtrees
  may differ in depth.
- Because `{folder}` names may not collide with kind names, any path is unambiguously
  parseable without knowing which variant it matched.
- The "no mixing" rule means a slice that grows a sub-slice must relocate its existing
  kind folders. This refactor is **intentional** — it is the upgrade path made visible.

### `folder1` vocabulary — rationale

"Screaming architecture": the top level should announce what the app *does*.

- `features` — where the bulk of the code goes
- `app` — composition root / entry points. Technically could be a feature, but calling
  it out makes the entry point visible at a glance.
- `pages` — per-page entry points for frontend apps
- `shared` — used by two or more features. Named explicitly so cross-cutting code has a
  legal home; without it people invent `features/common/`, a fake feature that becomes
  the junk drawer.

Unused entries are simply absent (`pages` in a Python backend). Deliberately excluded:
`lib` / `core` / `domain` (synonyms for `shared` or `features` — having both invites the
exact bikeshedding this tool removes), `utils`, and a top-level `types` (it's a kind, not
a grouping; global type dumps are how slices start leaking into each other).

### `tests`

`tests` is a root that *may* use the same grammar, but is **not** required to mirror the
`src` tree. The tool never checks correspondence ("every module has a test") — that is
coverage tooling's job.

No tests-specific content rules are needed. `test_login_succeeds.py` containing exactly
`def test_login_succeeds` satisfies the ordinary naming rule, and pytest's discovery glob
is satisfied for free. `conftest.py` is a default exception.

Consequence: many small files, so `@pytest.mark.parametrize` becomes essential for
covering a table of cases in one function.

## Layer 2 — file naming

- Filename is the `snake_case` of the module's single public name.
  `functions/authenticate.py` → `def authenticate`;
  `types/credentials.py` → `class Credentials`.
- Where `single_public_name` is waived (e.g. `constants/`), there is nothing to derive a
  name from, so `filename_matches_public_name` **deactivates automatically** and only the
  casing check applies. Such files are free-named topic labels: `constants/http.py`.

## Layer 3 — content

### Kinds are derived from the language, not from architecture

The key move: don't ask "what architectural category is this," ask **"what does the
module's public name denote?"** There are only three answers a program can give.

| denotes | kind | Python |
|---|---|---|
| a **callable** | `functions/` | `def`, `async def` |
| a **type** | `types/` | `class`, `type X = ...`, `NewType(...)` |
| a **value** | `constants/` | anything else bound at module level |

`constants` needs no positive detection — it is the **else branch**. A module-level
binding that is neither a `def` nor a type denotes a value.

This ports cleanly:

- **Rust** — `fn` / `struct`,`enum`,`trait`,`type` / `const`,`static`
- **TypeScript** — `function` + `const f = () => …` / `type`,`interface`,`class` / other `const`
- **C++** — functions / `class`,`struct`,`enum`,`using` / `constexpr`,`const`

### Rules

1. `single_public_name` — exactly one top-level binding without a leading underscore.
   Unlimited `_private` helpers and imports are allowed alongside it.
2. `filename_matches_public_name` — see layer 2.
3. `kind_matches_declaration` — the public name's declaration category matches its
   kind folder.

Rule 3 subsumes what would otherwise be a separate "type alias spelling" rule: a bare
`X = int` in `types/` denotes a *value*, so the diagnostic is
*"this denotes a value — move it to `constants/`, or write `type X = int`."*

### Rules deactivate when what they depend on is waived

A rule with nothing left to check switches itself off; it never has to be listed in a
`waive` array.

- `single_public_name` waived → `filename_matches_public_name` has no name to derive from,
  so only the casing check applies (this is how `constants/` files become free-named topic
  labels).
- `file_must_be_in_kind_folder` waived → there is no kind folder to compare against, so
  `kind_matches_declaration` deactivates (this is how `{root}/app/**` works).

### What counts as a public name (Python)

> Count top-level `def` / `class` / `type` / assignment bindings whose name does not start
> with `_`, dedupe by name, skip imports and conditional blocks. Expect exactly one.

The edge cases, each decided deliberately:

- **Imports are excluded.** `from datetime import datetime` binds a module-level
  `datetime` with no underscore, so a naive count would flag every file that imports
  anything. Covers `try: … except ImportError:` fallback imports too.
- **`@overload` dedupes by name.** Three `def process` stubs plus an implementation are
  four AST nodes and one public name.
- **`if TYPE_CHECKING:` is skipped entirely.** The block is `False` at runtime, so nothing
  defined inside it exists when the module is imported — it is not API surface. (A type
  checker treats the block as always-true, so importing such a name passes checking and
  fails at runtime; a known footgun, not a reason to count it.) In practice the block
  holds only imports, which are excluded anyway.
- **Dunders are excluded** — `__version__`, `__author__`, `__all__`.
- **`__all__` is ignored as a signal.** The underscore convention is the single source of
  truth. Honoring both means two mechanisms that can contradict each other
  (`__all__ = ["foo"]` beside a public `bar`).

### Deliberately not implemented

Ruff/ESLint already do these well, and duplicating them means two tools disagreeing:
wildcard imports, unused imports, in-file naming casing, docstring presence, annotation
coverage. Also excluded: import-direction rules (see scope note), module side effects,
`__init__.py` re-export policy.

### Python version

Targets **3.12+**, which makes the type layer mechanically checkable:

- `type X = int` — PEP 695 gives aliases an actual keyword
- `T = TypeVar("T")` / `ParamSpec` — these **cease to exist** as module-level names;
  PEP 695 inlines them (`def call[T, **P](...)`, `class Box[T]`)
- Functional forms rewrite to `class` and land in `types/` anyway
  (`TypedDict(...)` → `class Movie(TypedDict)`, likewise `namedtuple`, `Enum`)
- `NewType` is the one genuine straggler — it stays an assignment because it is *not*
  an alias (nominal, not interchangeable), but its RHS is a trivially detectable call

## Config

Zero config by default. The file is **`app-organizer.toml`** at the repo root — a
dedicated file, not a section in `pyproject.toml`, because the tool is polyglot (a Rust-only
repo has no `pyproject.toml`, and a `[tool.app_organizer]` block would be an odd home for
the TypeScript profile). Its presence also marks the project root for the walker.

**Format: TOML.** YAML reads better for this shape — the exceptions list especially — but
loses on two practical points. `{root}` starts a flow mapping in YAML, so every pattern
string would need quoting, and patterns are the most-edited field. And the Rust ecosystem
is weaker: `serde_yaml` was archived in 2024, leaving forks, whereas `toml` + `serde` is
the maintained default. TOML's verbosity is acceptable for a file that is, by design,
rarely edited.

**Rules are hard errors.** No severity levels — exceptions are the only escape hatch.
Comment-based inline overrides may come later; they are not needed now.

### Roots map to languages explicitly

```toml
[roots]
src   = "python"
tests = "python"
web   = "typescript"
```

Each language declares its own roots, and roots may not overlap. Keying the map *by root*
makes that unrepresentable rather than merely validated — a duplicate TOML key is a parse
error. It also gives the walker trivial dispatch: match the first path component, pick the
profile.

Default with no config: `src` and `tests` → python. A TypeScript-only repo writes two
lines; everyone else writes nothing.

**Rejected: inferring the language per root from file extensions.** Attractive for
zero-config (a mixed root would just be a hard error), but it accumulated special cases
faster than it was worth. Extension lists survive, demoted from *inference* to
*validation*: a `.rs` file under a root declared python is a hard error.

Each profile carries a hardcoded extension list:

```
python:     .py .pyi
typescript: .ts .tsx
rust:       .rs
cpp:        .cpp .cc .hpp .h
```

Extensions outside every list are **untracked** — invisible to the tool, governed by
nothing. A `README.md` or `fixtures.json` may sit anywhere. This keeps the tool a *code*
organizer; policing `.png` placement would invite a long tail of exceptions that dilute
the zero-config promise.

### Profile

```toml
[python]
kinds = ["functions", "types", "constants"]

patterns = [
  "{root}/{folder1}/{folder2}/{folder3}/{kind}/{files}",
  "{root}/{folder1}/{folder2}/{kind}/{files}",
  "{root}/{folder1}/{kind}/{files}",
]

[python.segments.folder1]
one_of = ["app", "features", "pages", "shared"]

[python.segments.folder2]        # definitions are shared, referenced by name
not_one_of = "@kinds"
casing     = "snake_case"

[python.segments.folder3]
not_one_of = "@kinds"
casing     = "snake_case"

[python.segments.kind]
one_of    = "@kinds"
leaf_only = true                 # files only, never subfolders

[python.segments.files]
casing = "snake_case"
```

Segment definitions live apart from the patterns that use them, so a segment appearing in
several variants is defined once. Profiles do **not** define `root` — that comes from the
`[roots]` map, which keeps profiles portable between projects.

### Exceptions are scoped rule waivers

Not a path allowlist — each entry names a glob and the rules waived under it.

```toml
[[python.exceptions]]
path   = "**/constants/*.py"
waive  = ["single_public_name"]
reason = "constants files group related values by topic"

[[python.exceptions]]
path   = "**/__init__.py"
waive  = ["file_must_be_in_kind_folder", "single_public_name"]
reason = "Python requires package markers at every level; they re-export"

[[python.exceptions]]
path   = "**/py.typed"
waive  = ["file_must_be_in_kind_folder"]
reason = "PEP 561 requires this exact path"

[[python.exceptions]]
path   = "tests/**/conftest.py"
waive  = ["file_must_be_in_kind_folder", "single_public_name"]
reason = "pytest requires this exact filename for fixture discovery"

[[python.exceptions]]
path   = "**/__main__.py"
waive  = ["file_must_be_in_kind_folder", "single_public_name"]
reason = "Python requires this exact filename for `python -m pkg`"

[[python.exceptions]]
path   = "{root}/app/**"
waive  = ["file_must_be_in_kind_folder"]
reason = "the composition root wires things together; kinds add nothing there"

# user-added, not a default
[[python.exceptions]]
path   = "tests/**"
waive  = ["single_public_name"]
reason = "prefer classic multi-test modules"
```

- **User exceptions merge with the defaults**, they do not replace them — nobody should
  have to re-declare the `__init__.py` waiver. A `disable_default_exceptions` escape hatch
  can be added if removing a default ever becomes necessary.
- `reason` is **required** on user-defined exceptions — one line that turns every escape
  hatch into self-documenting history.
- `file_must_be_in_kind_folder` is the layer-1 rule that files appear only at `{files}`.
  It is by far the most-waived rule: every Python special filename needs it.
- **Globs may use pattern placeholders.** `{root}` expands to whatever the `[roots]` map
  declares for that profile. This matters for the `app` default: hardcoding `src/app/**`
  would break a repo whose root is `source/`, and `**/app/**` would wrongly match an
  ordinary `folder2` that happens to be named `app`.
- Defaults must be printable (`app-organizer defaults`) so "zero config" never means
  "opaque."

### `app` is kind-free but not name-free

`{root}/app/**` waives only `file_must_be_in_kind_folder`, so the composition root may be
a flat pile of files — and `kind_matches_declaration` deactivates with it. But
`single_public_name` still applies: `app/create_app.py` defines `create_app`,
`app/settings.py` defines `settings`. Glue code gets to skip the taxonomy; it does not get
to skip being one thing per file.

## Walking

Uses the `ignore` crate (the one ripgrep uses) so `.gitignore` is respected — "is this
file part of the project" is exactly what git already knows — and directory traversal is
parallel for free. Matching ripgrep's semantics means users already know how it behaves.

Fallback skip list when there is no `.gitignore`:

```
.git/  .venv/  venv/  node_modules/  target/  __pycache__/
```

The extension filter cannot replace this list, because the worst offenders are full of
*tracked* extensions — `.venv/` holds thousands of `.py` files, `node_modules/` thousands
of `.ts`. Caches holding only untracked extensions (`.mypy_cache/`, `.ruff_cache/`) are
filtered anyway and need no entry.

## Diagnostics

The tool's entire user interface is its error output, so it gets designed rather than
defaulted.

- **Category tags**, not numbered codes: `error[folder]`, `error[content]`, `error[kind]`,
  `error[root]`.
- **Print the tried patterns** on a folder failure. This is precisely what the flat-list
  format buys over a schema, and it teaches the convention at the moment someone breaks it.
- **Compute the suggested fix.** With no `--fix`, and agents doing remediation, a precise
  target path is worth a lot — and for a kind mismatch the tool knows it exactly.

```
error[folder]: no pattern matched
  --> src/features/auth/login/handlers/submit.py
   |
   | `handlers` is not a kind folder
   | expected one of: functions, types, constants
   |
   = tried: {root}/{folder1}/{folder2}/{folder3}/{kind}/{files}
            {root}/{folder1}/{folder2}/{kind}/{files}
            {root}/{folder1}/{kind}/{files}

error[content]: file declares 2 public names, expected 1
  --> src/features/auth/functions/authenticate.py:14
   |
   | public names: authenticate, validate_token
   | rename `validate_token` to `_validate_token`, or move it to its own file

error[kind]: `Credentials` denotes a type but lives in functions/
  --> src/features/auth/functions/credentials.py:3
   |
   = move to src/features/auth/types/credentials.py

error[root]: `src/` is declared python, but contains rust files
  --> src/features/geometry/functions/intersect.rs
```

## CLI

- `app-organizer check <path>` — diagnostics, non-zero exit on failure
- `app-organizer defaults` — print the effective config
- `--format json` alongside human-readable text, for wrapper packages to consume
- **No `--fix`.** Renaming is safe-ish, but *moving* a file between kind folders breaks
  every import of it. AI agents handle the remediation in most workflows today, so
  check-only is the honest v1.

## Follow-up

Once this tool exists, the folder-convention and file-naming sections of the user's
global `CLAUDE.md` should be deleted — the tool supersedes them.

## Still open

- Crate / binary naming and the repo layout for wrapper packages
- Build order for v1 — which rules land first, and what the test corpus looks like

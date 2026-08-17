# app-organizer — design

An opinionated, multi-language validator for **app folder conventions, file naming
conventions, and file content conventions**.

Zero config is the selling point: like Prettier, it should "just work" with sensible
locked-in defaults. An optional config file exists for the few things that genuinely
vary between projects.

**The thesis, in four words: one substantial export per file.**

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

**Language support:** Python and Rust ship. TypeScript (React UIs) and C++ follow.

---

# The rules

Seven rule names appear in diagnostics and in `waive` arrays, but there are only four
ideas.

| rule | idea |
|---|---|
| `single_primary_export` | at most one substantial export per file |
| `filename_matches_export` | the filename is that export's name |
| `max_file_lines` | a file with one stays under the budget |
| `folder_depth` | folders nest at most 3 below a root |
| `name_casing` | one casing per language, for every folder and file |
| `root_language_match` | a tracked file's language matches its root's declaration |
| `file_is_readable` | the tool could open and parse it at all |

## 1. At most one substantial export per file

Each profile names the **governed constructs** — the ones the language holds to one per
file:

```toml
[rust]    one_per_file = ["fn", "struct", "enum", "trait"]
[python]  one_per_file = ["def", "class"]
```

The engine carries **no language vocabulary at all**. `PublicName` has a
`construct: &'static str` that the profile fills with one of its own keywords, and the
core only ever asks whether the configured list contains it. That is the same principle
already applied to path segments — positional, not semantic, so each profile supplies
its own words.

### *At most* one, not exactly one

Zero governed exports is a **legal, deliberate shape**, and making it legal is the single
highest-leverage decision in the design. It derives away nearly every special case a
filename-based tool would otherwise need:

| file | governed exports | what a v1-style tool needed | v2 |
|---|---|---|---|
| `limits.rs` — five `pub const` | 0 | a `**/constants/*` glob | derived |
| `mod.rs`, `lib.rs`, `__init__.py`, `index.ts` | 0 | a glob each | derived |
| `auth.rs` beside `auth/` (2018 style) | 0 | a sibling-folder predicate in the engine | derived |
| `main.rs` — `fn main` is bare, so private | 0 | a glob | derived |
| `tests/cli.rs` — bare `#[test] fn` | 0 | a glob | derived |
| `stray.rs` — three `pub fn` | 3 | error | error |

The exception list stops growing linearly with the number of languages, which was the
real design pressure — not any one language's quirk.

### What counts as an export

**Rust: any visibility modifier** — `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`. A
bare item is private, the exact analogue of Python's `_helper`, except the compiler
enforces it.

`pub(crate)` counts. The rule is about one substantial thing per file, and `pub(crate)`
*is* the module's surface to the rest of the crate; excluding it would gut the rule for
exactly the kind of lib+bin crate this one is, where almost nothing is bare `pub`. Unlike
Python's advisory underscore this reads a real language construct, so the two profiles are
not answering quite the same question — an improvement, not a compromise.

Skipped entirely: `mod`, `use`, `impl`, `extern crate`, attributes. A `mod` names a file
rather than a thing; the rest either re-export or attach to something already counted.
Counting `impl` blocks would make every type's own file illegal.

`macro_rules!` takes no visibility modifier and leaves the crate only via
`#[macro_export]`, which tree-sitter-rust hangs as a **preceding sibling** rather than a
child — the one place Rust's tree does not nest what reads like it should. It is
ungoverned by default, so this currently changes no verdict; it is implemented because a
project may add `"macro_rules"` to `one_per_file`.

**Python: top-level and not `_`-prefixed.** The language has no visibility keyword, so the
advisory convention is the only signal there is. That excludes dunders (`__version__`,
`__all__`) for free. The other edge cases, each decided deliberately:

- **Imports are excluded.** `from datetime import datetime` binds a module-level
  `datetime` with no underscore, so a naive count would flag every file that imports
  anything. Covers `try: … except ImportError:` fallback imports too.
- **`@overload` dedupes by name.** Three `def process` stubs plus an implementation are
  four AST nodes and one export.
- **`if TYPE_CHECKING:` is skipped entirely.** The block is `False` at runtime, so nothing
  defined inside it exists when the module is imported — it is not API surface.
- **`__all__` is ignored as a signal.** The underscore convention is the single source of
  truth. Honoring both means two mechanisms that can contradict each other
  (`__all__ = ["foo"]` beside a public `bar`).

### Rejected: exactly one public name (the v1 rule)

v1 required exactly one top-level public binding of any kind. That forces one-line files
for constants and type aliases. Defensible, and pleasant for some, but too jarring a
default to ship — and the workaround it needed (`**/constants/*.py` waiving the rule) was
a folder-shaped patch on a construct-shaped problem.

### Rejected: a `linkage_files` whitelist

An overridable per-language list of filenames allowed to hold re-exports —
`["mod.rs", "lib.rs", "__init__.py", "index.ts"]`. Explored seriously, then retired: it
cannot express `auth.rs` (named after its module, not a fixed string), and as an
*addition* it would be a second mechanism overlapping `exceptions`, which is the "two
mechanisms that can contradict each other" trap this document already invokes when
rejecting `__all__` as a signal. "Zero governed exports is legal" covers every case it
would have.

## 2. The filename is that export's name

Transformed into the language's casing, **never copied**: `pub struct HTTPClient`
prescribes `http_client.rs` whatever the export happened to be called.

Because it is a transform and not the identity, a badly cased export cannot leak into a
filename — it is laundered. That is what lets this tool leave export naming to each
language's own toolchain, which is where it belongs:

| language | regulates export names | on by default |
|---|---|---|
| Rust | `rustc` — `non_snake_case`, `non_camel_case_types`, `non_upper_case_globals` | **yes** |
| Python | ruff `pep8-naming`, pylint `invalid-name` | pylint yes, ruff opt-in |
| TypeScript | `@typescript-eslint/naming-convention` | no |
| C++ | clang-tidy `readability-identifier-naming` | no |

### One casing per language

`name_case` is a single value governing **folder names, the filenames of files with no
governed export, and the export-name transform**. Not a list, not a per-construct matrix.

- **Rejected: a list of allowed casings.** TypeScript is the motivating case — PascalCase
  components, camelCase hooks. But allowing both means `Button.tsx` and `button.tsx`
  coexist in one repo, which enforces nothing.
- **Rejected: `preserve`** (mirror the export's own spelling). Handles the React mix
  elegantly, and is the one setting that lets a badly cased export leak into a filename.
  It would make our correctness depend on a linter the project may not have enabled.
- **Rejected: a per-construct transform** (PascalCase for types, camelCase for functions).
  Breaks immediately — React components are functions named in PascalCase.

One lowercase-family casing resolves all of it, and `button.tsx` for
`export function Button` is well-precedented: shadcn/ui ships `ui/button.tsx` and
`hooks/use-toast.ts`, the Next.js app router mandates lowercase special files, Angular has
always been kebab. It also sidesteps a real bug class — macOS and Windows are
case-insensitive, so `Button.tsx` and `button.tsx` collide, and git handles case-only
renames badly.

The transforms compose correctly: `Button` → `button`, `useAuth` → `use-auth`,
`HTTPClient` → `http-client`.

Only `snake_case` and `kebab-case` implement `suggest`. A casing with no converter can
prescribe nothing and stays quiet rather than guessing.

## 3. Files with a substantial export stay under 200 lines

Non-blank, non-comment lines, one number for every language. Comment nodes come free from
the tree-sitter parse already being performed, so a `#` inside a string literal is code
and a trailing `// note` still leaves its line counted — none of which a regex gets right.

This deliberately overlaps `pylint`'s `too-many-lines` and ESLint's `max-lines`, which the
*Deliberately not implemented* section below would normally argue against. **The overlap
is the point**: the value is a single threshold holding across four languages regardless
of which per-language linters a project has enabled, and clippy has no equivalent at all.

### The budget applies only to files with a governed export

The goal is that the functions and classes carrying an application's load-bearing logic
stay easy to read and browse. Types, constants, config tables, and other declarative files
can be whatever length their author wants — a 900-line lookup table or colour palette is
not improved by being split.

This yields a clean symmetry: **a governed export is what activates both extra
standards.** A file with one must be named after it and stay under the budget; a file
without one is free on both counts. Mechanically it is a second edge in the existing
deactivation cascade rather than a special case:

```
filename_matches_export -> single_primary_export
max_file_lines          -> single_primary_export
```

So waiving `single_primary_export` on a legacy tree lifts the line cap with it, which is
the desired behaviour anyway.

**Known hole, documented rather than fixed:** a file of purely *private* functions has
zero governed exports and is therefore uncapped. In Rust that is dead code and rustc's
`dead_code` lint catches it. In Python it is reachable — a `_helpers.py` of thirty
`_functions` imported across the app would dodge the budget. Getting there takes
deliberate effort.

### Rust's `#[cfg(test)] mod tests` does not count

Rust is the only language here with no convention for putting unit tests in another file,
and the compiler strips these blocks from what the file ships. The budget measures shipped
code, so taxing a crate for testing the idiomatic way would be measuring the wrong thing.
Python and TypeScript already put tests in separate files, so this keeps the rule *fair*
across languages rather than special-casing one.

## 4. Folders nest at most 3 deep below a root

That is the entire folder grammar. Where a file sits is up to the project; how far down it
sits is not. Three is the direct translation of v1's three free folder levels.

Only the shallowest offenders are reported — a tree two levels too deep would otherwise
produce a diagnostic for every directory beneath the first, all fixed by the same move.

A root's own name is the user's declaration and is not graded: `src/MyPackage` is theirs
to spell.

### There is no mandated top-level vocabulary

v1 required `folder1` to be one of `app` / `features` / `pages` / `shared` —
"screaming architecture", the top level announcing what the app *does*. It is dropped as a
rule for two reasons:

1. **It cannot be right for everyone.** The tool cannot know what a given project
   legitimately needs at that level.
2. **It was the largest source of false positives.** On the one real project this was
   shaken down against, it produced 19 identical diagnostics — which is what forced
   multi-component roots into existence.

Direct evidence, from this crate: reorganising it to satisfy its own rules produced
`config/`, `rules/`, `lang/`, `walk/`, `diagnostics/`, `engine/` — topical, not
`app`/`features`/`shared`. A library legitimately differs from an application.

It survives as **guidance**: `app-organizer init` writes the suggested tree into the
seeded config as a comment, and the README shows it.

---

# What v1 was, and why it changed

The v1 centrepiece was **kind folders**. Rather than ask "what architectural category is
this", it asked what a module's single public name *denoted* — and there are only three
answers a program can give:

| denotes | kind | Python |
|---|---|---|
| a **callable** | `functions/` | `def`, `async def` |
| a **type** | `types/` | `class`, `type X = ...`, `NewType(...)` |
| a **value** | `constants/` | anything else bound at module level |

It ported cleanly to Rust, TypeScript, and C++, and it was genuinely elegant. It is gone.

## The trigger: `foo.rs` beside `foo/`

Rust declares a submodule two ways — `src/features/auth.rs` sitting beside
`src/features/auth/`, or `src/features/auth/mod.rs`. Both mean `crate::features::auth`,
and converting between them breaks no imports. The 2018 style is the one rustfmt and the
edition guide recommend.

The v1 grammar allowed files at exactly one position — the `{files}` leaf under a kind
folder — so `auth.rs` was illegal. It tripped exactly one rule
(`file_must_be_in_kind_folder`; `no_mixed_children` and `kind_folder_is_leaf` compare
*directories* only, and files never entered that map).

**No glob could waive it.** An exception matches path text. `auth.rs` is not a fixed
filename like `__init__.py`; it is named after its module, so the only glob that covers it
is `{root}/**/*.rs`, which also legalises `src/features/stray.rs` — every stray file in
the tree. Permitting the pattern honestly required a new predicate in the engine: *does a
sibling folder share my stem?*

## Pulling the thread

Adding that predicate would have been maybe thirty lines. Asking whether it was worth it
turned up something better. Dropping kind folders dissolves:

- the sibling predicate, and with it `auth.rs`;
- `mod.rs`, which had no kind folder to sit in;
- whether `pub` replaces Python's underscore convention *for placement purposes*;
- whether Rust needs its own kind vocabulary.

Four Rust problems, one deletion. And the exception list — the thing that grows linearly
with every new language — collapses with it.

**The seam halves.** A language profile used to answer two questions: what are this
module's public names, and does each denote a callable, a type, or a value. Now it answers
one: what does this module export, and by which construct. The Rust profile is ~120 lines
including its unit tests; TypeScript and C++ get correspondingly cheap. That is what makes
the multi-language goal real rather than aspirational.

## What was given up

Plainly: the denotation→folder mapping was this document's declared centrepiece, and
nothing now guarantees that all of a slice's types are browsable in one place.

The trade is accepted because the surviving rules deliver most of the daily value at a
fraction of the cost — and because kind folders were also the reason the tool could never
offer `--fix`. Moving a file between them breaks every import of it; renaming one does
not.

## What was deleted

| deleted | why |
|---|---|
| `file_must_be_in_kind_folder` | files live anywhere |
| `kind_matches_declaration` | already cascaded off it |
| `no_mixed_children` | had no kinds to compare, and `auth.rs` beside `auth/` must be legal |
| `kind_folder_is_leaf` | no kind folders |
| `Denotation` and its three-way map | nothing maps a name to a folder |
| `kinds`, `@kinds`, `NameSet`, `not_one_of` | the indirection existed for kinds |
| `patterns`, `segments`, `SegmentRule` | patterns degenerate to a depth cap |
| `src/grammar/` (251 lines) | nothing left to match |
| `PublicName.type_alias_hint` | advice about `types/` placement |
| `{root}/app/**` exception | waived a rule that no longer exists |
| `**/py.typed` exception | already provably inert — `.typed` is not a tracked extension |
| `**/constants/*.py` exception | derived: no `def`, no `class`, no rule |
| `tests/**/conftest.py` exception | subsumed by a broader `tests/**` waiver |

Nine rules became seven. Both renamed rules changed meaning, so the rename is deliberate:
a stale `waive = ["single_public_name"]` must fail loudly at config load rather than
silently mean something new. The `Rule` enum already rejects unknown names.

## Rejected pattern-grammar alternatives (retained history)

These were the arguments for the flat pattern list over a schema. The list itself is gone,
but the reasoning shaped what replaced it and is worth keeping.

**Rejected: alternation syntax.** A single pattern with inline alternation is a mini-regex
language, requiring grouping, precedence, and error messages that explain *which branch*
failed. Three plain strings needed none of that, and they were greppable.

**Rejected: JSON-Schema-style nested config.** Explored at length, because it was more
precise than the flat list and genuinely *expressed* rules the flat version had to check
separately. Two encodings were sketched: a nested `oneOf` tree, and
directory-tree-as-document (feed the repo in as nested objects with `null` file leaves and
validate with a stock validator). The second one works — because `oneOf` applies to a
directory *object as a whole*, per-directory homogeneity becomes structural rather than a
separate check, and depth limiting falls out as `$def` chaining.

*Why rejected: diagnostics.* JSON Schema validators report `oneOf` failures as "does not
match exactly one subschema" plus a JSON Pointer. This tool's entire output *is* its error
messages, so that is the wrong thing to trade away. Producing decent messages would mean
walking the tree by hand anyway — leaving the validator doing the easy half while we write
the hard half.

Both objections apply with more force now: what replaced the pattern list is an integer.

---

# Config

Zero config by default. The file is **`app-organizer.toml`** at the repo root — a
dedicated file, not a section in `pyproject.toml`, because the tool is polyglot (a
Rust-only repo has no `pyproject.toml`, and a `[tool.app_organizer]` block would be an odd
home for the TypeScript profile). Its presence also marks the project root for the walker.

**Format: TOML.** YAML reads better for this shape but loses on two practical points.
`{root}` starts a flow mapping in YAML, so every glob would need quoting. And the Rust
ecosystem is weaker: `serde_yaml` was archived in 2024, leaving forks, whereas
`toml` + `serde` is the maintained default.

**Rules are hard errors.** No severity levels — exceptions are the only escape hatch.

## Roots map to languages explicitly

```toml
[roots]
src   = "python"
tests = "python"
web   = "typescript"
```

Each language declares its own roots, and roots may not overlap. Keying the map *by root*
makes duplicates unrepresentable rather than merely validated — a duplicate TOML key is a
parse error. It also gives the walker trivial dispatch: find the root a path sits under,
pick the profile.

### Roots may be more than one component deep

```toml
[roots]
"src/my_package" = "python"
```

Any installable Python project has a package directory, and nothing can be done about it:
`import features.auth` does not work, the distribution needs `my_package` to be the
importable name. Absorbing that level into the root, rather than adding a `{package}`
segment, is the right shape because it is *true* — the package directory **is** the root
of the source tree. Everything above it is packaging scaffolding, not application
structure, and the tool has no business grading it.

The consequences follow from that same reading:

- **The longest declared root wins.**
- **Roots may not overlap**, and this is checked: declaring both `src` and
  `src/my_package` is a hard error. Longest-match would silently pick one, and "which
  profile governs this file" should never be a tiebreak.
- **`{root}` in an exception glob expands to the whole root**, so `{root}/lib.rs` becomes
  `src/my_package/lib.rs`.
- **Files above the root are invisible**, so `src/setup_helpers.py` is governed by nothing.

**Rejected: inferring the language per root from file extensions.** Attractive for
zero-config, but it accumulated special cases faster than it was worth. Extension lists
survive, demoted from *inference* to *validation*: a `.rs` file under a root declared
python is a hard error.

```
python:     .py .pyi
typescript: .ts .tsx
rust:       .rs
cpp:        .cpp .cc .hpp .h
```

Extensions outside every list are **untracked** — invisible, governed by nothing. This
keeps the tool a *code* organizer; policing `.png` placement would invite a long tail of
exceptions that dilute the zero-config promise.

## Profile

Five values.

```toml
[rust]
one_per_file     = ["fn", "struct", "enum", "trait"]
max_file_lines   = 200
max_folder_depth = 3
name_case        = "snake_case"
```

Profiles do **not** define `root` — that comes from the `[roots]` map, which keeps
profiles portable between projects. A language with no built-in profile still starts from
a shared baseline, so `[typescript] name_case = "kebab-case"` is a one-line declaration
rather than a full profile.

`one_per_file` is the one field with no cross-language baseline; the other three are the
tool's position and are the same wherever they ship.

`union` is left out of Rust's list because a file built around one is already unusual;
`type`, `const` and `static` are left out for the same reason `const` is left out of
Python's. Any project can add them back.

## Exceptions are scoped rule waivers

Not a path allowlist — each entry names a glob and the rules waived under it.

```toml
[[python.exceptions]]
path   = "**/__init__.py"
waive  = ["filename_matches_export"]
reason = "Python requires package markers at every level"

[[python.exceptions]]
path   = "tests/**"
waive  = ["single_primary_export"]
reason = "a test module holds many test functions by design"

[[rust.exceptions]]
path   = "{root}/lib.rs"
waive  = ["filename_matches_export"]
reason = "cargo requires this exact filename"
```

After the v2 redesign the defaults reduce to **one shape**, plus one broad waiver:

> A file whose name the *language* dictates can never also be the name of its export, so
> requiring a match would be requiring the impossible.

That covers `__init__.py`, `__main__.py`, `mod.rs`, `lib.rs`, `main.rs`, and later
`index.ts`. The one waiver of a different kind is `tests/**` waiving
`single_primary_export`, because a test module holds many test functions by design — and
via the cascade that lifts the line budget for test files too.

- **User exceptions merge with the defaults**, they do not replace them. A file seeded by
  `app-organizer init` already contains the defaults verbatim, so appending is deduped by
  (path, waive) — `reason` is prose and does not count.
- `reason` is **required** — one line that turns every escape hatch into self-documenting
  history.
- Globs are compiled with `literal_separator(true)`, so `*` does not cross a `/`.
- Defaults must be printable (`app-organizer defaults`) so "zero config" never means
  "opaque." The whole default config now fits on one screen, which is itself a result.

## Rules deactivate when what they depend on is waived

A rule with nothing left to check switches itself off; it never has to be listed in a
`waive` array. This is expressed once as a property of the rule graph
(`Rule::depends_on`) rather than as special cases at each call site.

Both edges point at the same place, which is the v2 design in one line: a substantial
export is what activates the two extra standards.

---

# Walking

Uses the `ignore` crate (the one ripgrep uses) so `.gitignore` is respected — "is this
file part of the project" is exactly what git already knows. Matching ripgrep's semantics
means users already know how it behaves.

Fallback skip list when there is no `.gitignore`:

```
.git/  .venv/  venv/  node_modules/  target/  __pycache__/
```

The extension filter cannot replace this list, because the worst offenders are full of
*tracked* extensions — `.venv/` holds thousands of `.py` files, `node_modules/` thousands
of `.ts`.

# Diagnostics

The tool's entire user interface is its error output, so it gets designed rather than
defaulted.

- **Category tags**, not numbered codes: `error[content]`, `error[naming]`, `error[size]`,
  `error[folder]`, `error[root]`.
- **Compute the suggested fix.** With no `--fix`, and agents doing remediation, a precise
  target name is worth a lot.
- **One structural cause is grouped**, so a nesting cap blown across sibling subtrees
  prints one block with several paths rather than repeating the explanation.
- **When the content layer prescribes a filename, the casing rule stays quiet.** That
  rename fixes the casing too, and offering two different names for one file would be
  worse than saying nothing.

```
error[content]: file exports 2 substantial things, expected 1
  --> src/models.rs:5
   |
   | exports: struct User, enum Role
   | move all but one to files of their own, or make them private

error[naming]: file name does not match its export `class Credentials`
  --> src/features/auth/creds.py:1
   |
   = rename to credentials.py

error[size]: 209 lines of code, and the budget is 200
  --> src/tally.rs
   |
   | blank lines and comments are not counted
   |
   = pull a private helper out into its own file, or split the export in two

error[folder]: folders nest 4 deep below the root, and the limit is 3
  --> src/features/auth/login/forms
      src/features/billing/reports/quarterly
   |
   = flatten a level, or split the tree into two roots

error[root]: `src/` is declared python, but contains rust files
  --> src/features/geometry/intersect.rs
```

# Deliberately not implemented

Ruff/ESLint/clippy already do these well, and duplicating them means two tools
disagreeing: wildcard imports, unused imports, **in-file naming casing**, docstring
presence, annotation coverage. Also excluded: import-direction rules (see scope note),
module side effects, `__init__.py` re-export policy.

The line budget is the one deliberate exception, for the reason given above.

# CLI

- `app-organizer check <path>` — diagnostics, non-zero exit on failure
- `app-organizer defaults` — print the effective config
- `app-organizer init` — seed `app-organizer.toml` with the full defaults
- `--format json` alongside human-readable text, for wrapper packages to consume
- Exit codes: `0` clean, `1` violations, `2` the tool itself failed — a CI job wants to
  fail on 1 and shout about 2.
- **No `--fix` yet.** Now that files may live anywhere, a rename is the only fix the tool
  ever prescribes, which makes `--fix` genuinely feasible for the first time. It still
  needs the imports to follow, so it is future work rather than a v2 deliverable.

# Follow-up

Once this tool exists, the folder-convention and file-naming sections of the user's
global `CLAUDE.md` should be deleted — the tool supersedes them.

# Still open

- Whether **200** is calibrated correctly. Reorganising this crate to satisfy its own
  rules took the largest file from 315 code lines to 141, so nothing exceeds the budget
  today. It needs a larger corpus to tune against.
- Whether Python should count a file of purely private functions (see *Known hole*).
- TypeScript's `name_case` — `kebab-case` is sketched here and implemented in `Casing`,
  but no profile ships yet.
- Crate / binary naming and the repo layout for wrapper packages.

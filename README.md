# app-organizer

**One substantial export per file.**

Linters check the code *inside* your files. This checks where the files live,
what they are called, and whether each one is about one thing.

```
error[content]: file exports 2 substantial things, expected 1
  --> src/features/auth/authenticate.py:5
   |
   | exports: def authenticate, def validate_token
   | move all but one to files of their own, or make them private

error[naming]: file name does not match its export `class Credentials`
  --> src/features/auth/creds.py:1
   |
   = rename to credentials.py

error[size]: 209 lines of code, and the budget is 200
  --> src/features/billing/reconcile.rs
   |
   | blank lines and comments are not counted
   |
   = pull a private helper out into its own file, or split the export in two
```

Zero config by design — like Prettier, it should just work.

## Usage

```bash
app-organizer check .                 # diagnostics; exit 1 if any
app-organizer check . --format json   # same values, for tooling
app-organizer defaults                # print the effective config
app-organizer init                    # seed app-organizer.toml to edit
```

Exit codes: `0` clean, `1` violations found, `2` the tool itself failed.

## The conventions

Four rules, and the whole point is the first one.

### 1. At most one *substantial* export per file

Each language names the constructs it holds to one per file. Everything else
clusters freely:

| language | one per file | free to cluster |
|---|---|---|
| Python | `def`, `class` | constants, aliases, `TypeVar`s |
| Rust | `fn`, `struct`, `enum`, `trait` | `const`, `static`, `type`, `macro_rules!` |

*At most* one, not exactly one. **Zero is a legal, deliberate shape**: a
constants table, a module of type aliases, a `mod.rs`, an `__init__.py`, an
`index.ts`. Those files are free-named, and rules 2 and 4 do not apply to them.

Rust's `auth.rs` sitting beside `auth/` is legal for exactly this reason — it
declares submodules and re-exports, and neither counts as an export.

### 2. The filename is that export's name

Transformed into the language's casing, never copied: `pub struct HTTPClient`
prescribes `http_client.rs`. Because it is a transform, a badly cased export
cannot leak into a filename — so export naming stays the job of your own
toolchain (rustc's `non_snake_case`, ruff's `pep8-naming`,
`@typescript-eslint/naming-convention`, clang-tidy's
`readability-identifier-naming`).

Each language has exactly **one** casing, for every folder and file it owns —
not a list, and not a per-construct matrix. A list would let `Button.tsx` and
`button.tsx` coexist in one repo, which enforces nothing; a matrix breaks on the
first React component, which is a function named in PascalCase. It also
sidesteps a real bug class, since macOS and Windows are case-insensitive and git
handles case-only renames badly.

### 3. Folders nest at most 3 deep below a root

That is the whole folder grammar. Where files sit is up to you; how far down
they sit is not.

There is no mandated top-level vocabulary. `app-organizer init` scaffolds a
suggested tree, but a library legitimately organises by topic
(`config/`, `rules/`, `lang/`) where an application organises by feature, and
the tool cannot know which you are.

### 4. Files with a substantial export stay under 200 lines

Non-blank, non-comment, and deliberately overlapping `pylint`'s
`too-many-lines` and ESLint's `max-lines` — the value is *one* threshold holding
across every language you use, whichever per-language linters you happen to have
switched on. Clippy has no equivalent at all.

The budget rides on having a substantial export, so a 400-line colour table or
config map is nobody's business but its author's. Rust's `#[cfg(test)] mod
tests` blocks do not count either: the compiler strips them from what the file
ships, and Rust has no convention for putting unit tests anywhere else.

---

Files outside a declared root, and extensions no language claims, are invisible
to the tool: a `README.md` or `fixtures.json` may sit anywhere.

**`DESIGN.md` is the real specification**, including every rule, the config
format, and the alternatives that were rejected and why.

## Config

Everything is optional. The one thing most projects touch is which roots exist
and what language each holds:

```toml
[roots]
src   = "python"
tests = "python"
```

A root may be more than one component deep, which is how src-layout projects work —
the package directory *is* the root of the source tree, and the packaging
scaffolding above it is governed by nothing:

```toml
[roots]
"src/my_package" = "python"
tests            = "python"
```

Each language's profile is five values:

```toml
[rust]
one_per_file     = ["fn", "struct", "enum", "trait"]
max_file_lines   = 200
max_folder_depth = 3
name_case        = "snake_case"
```

`app-organizer init` writes the full defaults into `app-organizer.toml` so
overrides are edits to something visible rather than guesses at what is being
replaced. Anything deleted from that file falls back to its default —
`exceptions` are *added* to the defaults rather than replacing them, so a
default waiver cannot be switched off by deleting it.

Exceptions are scoped rule waivers, not a path allowlist:

```toml
[[python.exceptions]]
path   = "{root}/legacy/**"
waive  = ["single_primary_export"]
reason = "the legacy tree is frozen, not being reorganised"
```

Rules are hard errors; there are no severity levels, and exceptions are the
only escape hatch. `reason` is required, so every escape hatch documents itself.

Rules that lose what they depend on switch themselves off. Both
`filename_matches_export` and `max_file_lines` depend on
`single_primary_export`, so the waiver above lifts all three — which is the
design in one line: **a substantial export is what activates the two extra
standards.**

## Status

Python and Rust ship. TypeScript and C++ follow — the engine is
language-agnostic, and a language profile answers one question: what does this
module export, and by which construct?

There is no `--fix` yet. Renaming a file is safe-ish; it is still your imports
that have to follow.

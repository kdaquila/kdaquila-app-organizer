# app-organizer

Linters check the code *inside* your files. This checks where the files live,
what they are called, and whether each one is about one thing.

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

error[kind]: `Credentials` denotes a type but lives in functions/
  --> src/features/auth/functions/credentials.py:3
   |
   = move to src/features/auth/types/credentials.py
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

Three layers, all derived from one idea: a module is named after the single
public thing it exports, and *what that thing denotes* decides which folder it
belongs in.

| the public name denotes | kind folder | Python |
|---|---|---|
| a callable | `functions/` | `def`, `async def` |
| a type | `types/` | `class`, `type X = ...`, `NewType(...)` |
| a value | `constants/` | anything else bound at module level |

Files live at one of three depths, and a directory's children are all kinds or
all folders — never both:

```
src/{folder1}/{kind}/{file}.py
src/{folder1}/{folder2}/{kind}/{file}.py
src/{folder1}/{folder2}/{folder3}/{kind}/{file}.py
```

`folder1` is a closed set — `app`, `features`, `pages`, `shared` — so the top
level announces what the app *does*.

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

`app-organizer init` writes the full defaults into `app-organizer.toml` so
overrides are edits to something visible rather than guesses at what is being
replaced. Anything deleted from that file falls back to its default —
`exceptions` are *added* to the defaults rather than replacing them, so a
default waiver cannot be switched off by deleting it.

Exceptions are scoped rule waivers, not a path allowlist:

```toml
[[python.exceptions]]
path   = "tests/**"
waive  = ["single_public_name"]
reason = "prefer classic multi-test modules"
```

Rules are hard errors; there are no severity levels, and exceptions are the
only escape hatch. `reason` is required, so every escape hatch documents itself.

Rules that lose what they depend on switch themselves off: waive
`single_public_name` and there is no name left to derive a filename from, so
`filename_matches_public_name` deactivates too. That is how `constants/` files
get to be free-named topic labels.

## Status

Python ships first. TypeScript, Rust, and C++ follow — the engine is
language-agnostic, and a language profile answers only two questions: what are
this module's public names, and what does each one denote?

There is no `--fix`. Renaming would be safe-ish, but *moving* a file between
kind folders breaks every import of it.

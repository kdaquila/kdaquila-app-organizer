# Room algebra — does it survive scale?

A probe, not a product. It answers one question: when you classify every
construct by *how many rooms consume it*, does the result stay small enough to
read, and are the things it flags real?

## The rule under test

Attribute every declaration to a room (a folder), resolve every reference
through re-export facades to the declaration that actually owns it, then apply
three cases that exhaust a construct's *foreign* consumer rooms:

| foreign consumer rooms | verdict |
|---|---|
| ≥ 2, and home room is not a sink | **HOIST** — shared code in a room that also consumes |
| exactly 1 | **MOVE** — misfiled; it belongs where it is used |
| 0 | **OK** |

A *sink* is a room with no outbound room edges — a pure supplier. That is the
only place shared code is allowed to live.

## Results

Python 3.12, `--depth 1 --skip-tests`, `import x.y` pseudo-edges excluded.

| corpus | constructs | rooms | sinks | cross-room | **HOIST** | MOVE |
|---|---|---|---|---|---|---|
| pygments | 1 518 | 20 | 11 | 89 (5.9%) | **3 (0.20%)** | 62 |
| cloud-init | 2 338 | 42 | 10 | 118 (5.0%) | **21 (0.90%)** | 88 |
| twisted | 4 250 | 27 | 2 | 214 (5.0%) | **76 (1.79%)** | 137 |

Whole run: 2.3 s for all three.

### HOIST is the signal

It stays under 2% of constructs, and the top hits are the genuine shared kernel
of each codebase — found with no human input:

```
twisted    11 rooms  twisted.python.failure:Failure
           10 rooms  twisted.logger._logger:Logger
            8 rooms  twisted.internet.defer:Deferred
cloud-init  5 rooms  cloudinit.helpers:Paths
            4 rooms  cloudinit.subp:ProcessExecutionError
            3 rooms  cloudinit.sources:DataSource
pygments    2 rooms  pygments.lexer:Lexer
```

Two of these are real architectural facts, not restatements of "this is
popular":

- **pygments** — `pygments.lexer` fails the sink test on exactly one edge:
  `Lexer -> pygments.filters:get_filter_by_name`. Every other thing it touches
  (`util`, `token`, `filter`, `regexopt`) is a true sink. The abstract base
  class reaches into the concrete filter registry: a textbook dependency
  inversion, surfaced by three findings over 1 518 constructs.
- **twisted** — `twisted.python.log -> twisted.logger._global`, while
  `twisted.logger -> twisted.python`. A real cycle between the legacy logging
  shim and its replacement.

### MOVE is noise

62–137 findings per corpus, and it fires on nearly every construct used by
exactly one other room — which describes most ordinary cross-room usage. It
also drives the non-convergence below. Recommend dropping it, or restricting it
to constructs whose single consumer is in a *different* subtree.

### The fixpoint does not converge

Not on any corpus. All three reach a repeating state rather than zero:

```
pygments    (3,64) -> (4,17) -> (3,13) -> (3,13) -> (3,13)
twisted     (116,166) -> ... -> (117,75) -> (117,75)
cloud-init  (49,111) -> ... -> (27,21) -> (27,21)
```

Two causes:

1. HOIST sends constructs to a derived `room.__shared__`, but they carry their
   own dependencies, so the new room often is not a sink either and fires again.
2. MOVE can pull a construct out of a shared room that a later round wants back.

It also proliferates rooms rather than deriving a layering — twisted 27 → 42,
cloud-init 42 → 67. **"Run it to a fixpoint and it computes your architecture"
does not hold.** The plausible repair is to hoist in dependency order —
condense the room graph, process sinks-first, and never create a shared room
whose own dependencies are not already resolved — but that is untested here.

### Room granularity dominates the result

`--depth` changes the finding count by an order of magnitude:

| corpus | HOIST @ depth 1 | HOIST @ depth 2 |
|---|---|---|
| pygments | 3 | 35 |
| cloud-init | 21 | 51 |
| twisted | 76 | 159 |

At depth 2 these packages have 219–345 rooms for 1 500–4 250 constructs, which
is not a room, it is a file. So the analysis cannot pick its own granularity: it
needs the rooms declared, which is the floorplan the whole idea started from.

### Twisted's count is inflated, and that is itself the finding

Twisted has **2 sinks**, both trivial (`__main__`, `_version`). With no pure
supplier layer anywhere, "home is not a sink" is nearly always true and HOIST
degrades toward "shared by 2+ rooms." Precision at the top of the list is
excellent; the tail is padded. Read the top N, not the total.

## What this says about the idea

Keep: HOIST, ranked by consumer-room count, over human-declared rooms. Small,
fast, language-agnostic in principle, and it finds real inversions.

Drop: MOVE as specified, and the claim that iterating derives a layering.

Unresolved: hoisting in dependency order; whether a construct's *own*
dependencies should constrain where it may be hoisted to.

## Running it

```bash
cd probe/src
PYTHONPATH=. python3 -m room_algebra.main <package-dir>... --depth 1 --skip-tests
```

## Limits

Static imports only — no framework registries, DI, or dynamic dispatch, so
signal receivers and plugin hooks read as never-used. `import x.y` followed by
attribute access is captured at module granularity only. Python only.

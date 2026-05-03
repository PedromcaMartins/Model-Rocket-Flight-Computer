# `docs/` — Project-wide documentation

This folder is the home for documentation that is **not specific to one subsystem**: project goals, requirements, system-level architecture, cross-cutting decisions, progress, and media.

## Scope of this folder

`docs/` owns the architecture-and-interface side of the split:

- **Goals & non-goals** — what the project is and is not trying to do, with reasoning.
- **Constraints** — system-level rules (cost, mass, regulatory, methodology) and *why* they exist.
- **Requirements** — testable statements derived from the above.
- **System-level architecture** — how subsystems fit together; the interfaces between flight computer, simulator, and ground station.
- **ADRs** — decisions that affect more than one subsystem.
- **Progress & media** — status snapshots, photos, flight logs.

Detailed design lives next to its artifact: [`../code/README.md`](../code/README.md), [`../hardware/README.md`](../hardware/README.md), [`../structure/README.md`](../structure/README.md), [`../open rocket/README.md`](../open%20rocket/README.md).

## Current contents

| Path | Purpose |
|---|---|
| `how-we-work.md` | Project-wide policies: docs split, traceability, TODO placement. |
| `GLOSSARY.md` | Project vocabulary — read before writing prose. |
| `REQUIREMENTS.md` | Numbered, testable requirements (`[DEV-*]`, `[ROCKET-*]`, `[SW-*]`). The contract the system is built against. |
| `Architecture/` | Subsystem-level architecture documents (interfaces, goals/non-goals/constraints per subsystem). Start here. |
| `software.md` | *Superseded.* Old single-page block diagram, kept only as a pointer into [`Architecture/`](Architecture/). |
| `ADR/` | Architecture Decision Records — one file per decision, capturing context, options, and the chosen trade-off. |
| `TODO.md` | Pending work across the whole system. Crate-internal TODOs live with the crate. |

When this list goes stale, fix it.

## Writing new architecture docs

Use this skeleton for a new subsystem doc in `Architecture/`:

```markdown
# <subsystem name>

## Goals
- ...

## Non-goals
- ...

## Constraints
- <constraint> — *why:* <reasoning>

## Interfaces
- <peer subsystem> ← <what crosses the boundary, in what direction>

## Open questions
- ...
```

Use this skeleton for an ADR in `ADR/`:

```markdown
# ADR-NNNN: <decision title>

- **Status:** proposed | accepted | superseded by ADR-XXXX
- **Date:** YYYY-MM-DD

## Context
What forces are at play? What problem is this solving?

## Options considered
- Option A — pros/cons
- Option B — pros/cons

## Decision
The chosen option and *why*.

## Consequences
What this enables, what it costs, what becomes harder.
```

Number ADRs sequentially (`ADR-0001-…`, `ADR-0002-…`).

## See also

- [`GLOSSARY.md`](GLOSSARY.md) — project vocabulary.
- [`../README.md`](../README.md) — human-facing motivation, repo layout, and how the project is organized.
- [`../AGENTS.md`](../AGENTS.md) — instructions for AI agents (documentation maintenance protocol, agent-flavored cheat sheet).

# `docs/` — Project-wide documentation

This folder is the home for documentation that is **not specific to one subsystem**: project goals, requirements, system-level architecture, cross-cutting decisions, progress, and media.

## Scope of this folder

`docs/` owns the architecture-and-interface side of the split:

- **Goals & non-goals** — what the project is and is not trying to do, with reasoning.
- **Constraints** — system-level rules (cost, mass, regulatory, methodology) and *why* they exist.
- **Requirements** — testable statements derived from the above.
- **System-level architecture** — how subsystems fit together; the interfaces between flight computer, simulator, and ground station.
- **Specs** — interface contracts for cross-subsystem boundaries (what a boundary guarantees, not how it is built).
- **ADRs** — decisions that affect more than one subsystem.
- **Progress & media** — status snapshots, photos, flight logs.

Detailed design lives next to its artifact: [`../code/README.md`](../code/README.md), [`../hardware/README.md`](../hardware/README.md), [`../structure/README.md`](../structure/README.md), [`../open rocket/README.md`](../open%20rocket/README.md).

## Current contents

| Path | Purpose |
|---|---|
| `how-we-work.md` | Project-wide policies: docs split, traceability, TODO placement. |
| `GLOSSARY.md` | Project vocabulary — read before writing prose. |
| `REQUIREMENTS.md` | Numbered, testable requirements (`[DEV-*]`, `[ROCKET-*]`, `[SW-*]`). The contract the system is built against. |
| `software/` | Subsystem-level architecture documents (interfaces, goals/non-goals/constraints per subsystem). |
| `ADR/` | Architecture Decision Records — one file per decision, capturing context, options, and the chosen trade-off. |
| `TODO.md` | Pending work across the whole system. Crate-internal TODOs live with the crate. |
| `ROADMAP.md` | Planned host-stack milestones (tasks 1–7): proto feature gating → split binaries → TUI. Each task links to its ADR or spec. Read before picking up any host-stack work. |

When this list goes stale, fix it.

## Writing new architecture docs

Use this skeleton for a spec in `software/` (cross-subsystem) or `code/<crate>/` (single-crate):

```markdown
# <interface or component name> — Spec

- **Status:** draft | accepted | superseded
- **Date:** YYYY-MM-DD

## Purpose
What does this interface / component do? What guarantees does it provide?

## Scope
What is in scope? What is explicitly out of scope?

## Interface contract
<type definitions, trait signatures, message formats, protocol description, or prose>

## Invariants
What must always be true? What can callers assume?

## Open questions
- ...
```

A spec must exist (or be updated) before any architectural or interface change lands. A spec that diverges from the code is a bug.

Use this skeleton for a new subsystem doc in `software/`:

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

Number ADRs sequentially (`ADR-001-…`, `ADR-002-…`).

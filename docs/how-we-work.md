# How we work

Project-wide policies and conventions. These apply to every discipline and subsystem in the repo.

## Architecture & detailed design

Documentation in this project is split along one axis. The split applies to **every engineering discipline** here — software, electronics, mechanical, systems engineering — not just code.

| | **Architecture & interface design** | **Detailed design** |
|---|---|---|
| Lives in | `docs/` | next to the artifact (`code/<crate>/…`, `hardware/…`, `structure/…`, `open rocket/…`) |
| Answers | *What* the rocket / subsystem does and *why* | *How* it is built |
| Contains | Goals, non-goals, constraints (with reasoning), public interfaces between subsystems, system-level diagrams, requirements, ADRs scoped to architecture | Implementation specifics: chosen libraries / parts / footprints / materials, internal layout, build/test/fab recipes, implementation tasks |
| Audience | Anyone trying to understand or evolve the rocket | Anyone modifying that specific subsystem |
| Stable? | Changes deliberately; reviewed | Changes frequently with the artifact |

Rules of thumb:

- A **goal, non-goal, or constraint** belongs in `docs/`, regardless of discipline. State the *reason* alongside it — a constraint without reasoning rots the moment circumstances change.
- A **choice of part, crate, library, footprint, material, file layout, encoding format, or test harness** is detailed design. It belongs next to the artifact.
- A **public interface between two subsystems** (e.g. avionics electronics ↔ structure mounting, recovery ↔ flight-computer trigger signal) is architecture. Its *implementation* is detailed design.
- If you find detailed design in `docs/` or architecture inside `code/` / `hardware/` / `structure/`, move it.

When proposing a non-trivial change, sketch the architecture / interface impact in `docs/` first; sketch the detailed design in the affected subsystem's folder second.

## Traceability policy

Every requirement in this project must be traceable end-to-end:

| Stage | What | Where |
|---|---|---|
| **Creation** | Why does this requirement exist? | `docs/REQUIREMENTS.md` — `Rationale` field |
| **Constraint** | What does it prohibit or mandate for design? | `docs/REQUIREMENTS.md` — stated in the requirement body |
| **Decision** | How was it implemented or resolved? | ADR in `docs/ADR/` or subsystem README |
| **Verification** | How do we know it is met? | `docs/REQUIREMENTS.md` — `Verification` field |

Rules:
- Every requirement **must** have a `Rationale` (why it exists) and a `Verification` (how it is confirmed met).
- Every design decision in a subsystem **must** reference the requirement(s) it satisfies.
- A decision with no traceable requirement is incomplete — either identify the requirement it satisfies, or add a new one.
- "We always did it this way" is not a rationale. State the constraint or goal the requirement protects.

## TODO policy

All pending work lives in exactly one place:

| Kind | Where |
|---|---|
| **System-level** — spans multiple subsystems, or has no single subsystem home | [`docs/TODO.md`](TODO.md) |
| **Subsystem-internal** — scoped to one subsystem (`code/`, `hardware/`, `structure/`, `open rocket/`) | that subsystem's own `TODO.md` |

Within `code/`, per-crate TODOs (e.g. "switch this to `thiserror`", "make sensor task generic over device") belong in the crate's own `TODO.md`, not in `docs/TODO.md`. If a TODO outgrows its subsystem, promote it to `docs/TODO.md`.

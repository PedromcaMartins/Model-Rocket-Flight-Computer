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

## Spec & ADR policy

Two lightweight artifacts gate non-trivial changes:

| Artifact | Answers | Written when | Required for |
|---|---|---|---|
| **Spec** (interface contract) | *What* must this interface do? What does it guarantee? | Before changing or adding an interface | Any architectural or interface change |
| **ADR** (Architecture Decision Record) | *Why* this approach? What did we reject? | At or after the decision point | Any significant design decision — architectural or implementation-level |

Rules:

- **An architectural change requires a spec first.** "Architectural" means: a new or changed interface between subsystems, a new subsystem, or a change to a system-level constraint. A spec can be code (type definitions, trait signatures) or prose. It must exist — or be updated — before the implementation lands.
- **A significant decision requires an ADR.** "Significant" means: a choice between two or more real alternatives, or a decision that would surprise a future engineer. Trivial choices do not need ADRs.
- Both artifacts follow the same architecture / detailed-design split:

  | Scope | Spec lives in | ADR lives in |
  |---|---|---|
  | Cross-subsystem interface or decision | `docs/software/` (existing subsystem doc or new file) | `docs/ADR/` |
  | Single-crate / single-subsystem | `code/<crate>/README.md` or sibling file | inline in that crate's README or a sibling note |

- An ADR without a traceable requirement is incomplete — see the Traceability policy.
- A spec that no longer matches the code is a bug — update it when the interface changes.

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

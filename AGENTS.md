# AGENTS.md

Instructions for AI coding agents working in this repository. Read this file first; it is the entry point to the rest of the documentation.

## 1. Self-improving documentation

**Whenever you finish a task, ask three questions before reporting done:**

1. Did I learn something about the project (a goal, a constraint, an interface, a non-obvious decision) that is not yet written down? → Update the relevant doc.
2. Did the change I just made invalidate something that *is* written down (an architecture diagram, an interface contract, a folder layout)? → Update the relevant doc.
3. Did I find a doc that contradicts the code, is stale, or points at a moved/deleted file? → Fix it, or note it in `docs/TODO.md`.

If a doc was helpful, leave it better than you found it. If it was misleading, fix it before moving on. Treat documentation drift as a bug.

**Where to put new documentation** — see §5. If you cannot decide between two locations, pick one and add a one-line pointer from the other. Do not duplicate content.

## 2. Vocabulary

Project vocabulary lives in [`docs/GLOSSARY.md`](docs/GLOSSARY.md). Read it before writing prose for the repo.

A few terms are easy to mix up; do not blur them:

- **Avionics** is the rocket subsystem (on-board electronics + software, broadly). Inside it, the **flight computer** (a.k.a. **flight software**, abbreviated **FC**) is the software portion only — the firmware — and **avionics electronics** is the PCB / schematic / parts. The PCB is *not* the flight computer.
- **Rocket subsystem** ≠ **software subsystem** — they are different axes (see glossary).

## 3. Architecture vs. detailed design — by-discipline classifier

The architecture-vs-detailed-design philosophy (the table, the rules of thumb, when each applies) lives in [`docs/README.md`](docs/README.md). The Traceability Policy also lives there. Read both before classifying a piece of writing or writing a new requirement.

The list below is your quick reference for *which folder* matches *which discipline* on each side of the split:

- **Software architecture** (in `docs/`): the FC ↔ ground-station interface; the simulator ↔ FC interface; SITL/HITL goals & constraints.
- **Software detailed design** (next to code): which crates we use, how modules are organized, how we serialize on the wire, how tasks are scheduled.
- **Avionics electronics architecture** (in `docs/`): which buses cross the FC ↔ sensors boundary; power-domain partitioning; what the flight software is allowed to assume about the board.
- **Avionics electronics detailed design** (in `hardware/`): schematic, PCB layout, exact part numbers, footprint choices, decoupling strategy.
- **Mechanical architecture** (in `docs/`): airframe envelope, mass budget, recovery activation interface, mounting points exposed to avionics.
- **Mechanical detailed design** (in `structure/`, `open rocket/`): CAD geometry, fillet sizes, print orientation, simulation parameters.
- **Systems architecture** (in `docs/`): what the rocket as a whole must do; cross-discipline constraints (e.g. avionics must fit in fuselage of diameter X, motor class limits mass to Y).

When proposing a non-trivial change, sketch the architecture/interface impact in `docs/` first; sketch the detailed design in the affected subsystem's folder second.

**Before any architectural or interface change, a spec must exist (or be updated).** Before any significant design decision, an ADR must be written. Both follow the same architecture/detailed-design split as all other docs — see `docs/how-we-work.md` §*Spec & ADR policy* for the full rules.

**Tasks in [`docs/ROADMAP.md`](docs/ROADMAP.md) follow this same split.** Architectural constraints and cross-subsystem contracts go in `docs/software/spec.md`; detailed implementation lives in the crate's own code docs and `Cargo.toml`.
Do not create ADRs for tasks that are purely architectural but do not involve a significant decision (e.g. "add a new sensor to the bus") — the spec is the right place for that. 
Do create ADRs for architectural tasks that involve a non-obvious decision (e.g. "add a new sensor to the bus, and we have to choose between I2C and SPI") — the spec should link to the ADR, but the rationale and decision live in the ADR.
Do create ADRs for detailed-design tasks that involve a non-obvious decision (e.g. "we need to choose a new async runtime for the FC, and there are three contenders") — the rationale and decision live in the ADR, which is linked from the crate's README or code docs. 
Do not create ADRs for detailed-design tasks that are purely mechanical (e.g. "we need to choose a new async runtime for the FC, and we have already decided on Tokio") — the crate's README or code docs are the right place for that.

## 4. Project map

```
.
├── AGENTS.md              ← you are here
├── README.md              ← human-facing motivation + repo layout
│
├── docs/                  ← project-wide docs: goals, requirements, architecture, ADRs, progress, media
│   ├── README.md          ← architecture vs. detailed-design philosophy
│   ├── GLOSSARY.md        ← project vocabulary
│   └── toolchain.md       ← installed Rust tools & targets reference
│
├── code/                  ← Rust workspace (flight computer, ground station, proto, simulator, xtask)
│   └── README.md          ← crates overview + detailed-design index
│
├── hardware/              ← KiCad project, BOM, electronics design
│   └── README.md
│
├── open rocket/           ← OpenRocket simulation files
│   └── README.md
│
├── structure/             ← CAD models (FreeCAD/Fusion) for printable rocket parts
│   └── README.md
│
├── datasheets/            ← vendor datasheets for parts used (reference only)
├── papers/                ← research papers (reference only)
├── gps_config/            ← u-blox GPS configuration artifacts
└── .cargo/ .vscode/ .zed/ ← tooling configs
```

Each "relevant" folder (`code/`, `hardware/`, `open rocket/`, `structure/`) has its own `README.md` that explains *what is in it* and *how it is organized*. Cross-cutting concerns live in `docs/`.

If you create a new top-level folder that is not pure reference material, add a `README.md` to it and link it here.

## 5. Where things go (cheat sheet)

| You want to write… | Put it in… |
|---|---|
| A new requirement (any discipline) | `docs/REQUIREMENTS.md` |
| A goal or non-goal for a rocket subsystem or software subsystem | `docs/` (existing or new architecture doc) |
| A system-level or cross-subsystem interface contract / diagram | `docs/` |
| A spec for a cross-subsystem interface | `docs/software/` (existing subsystem doc or new file) |
| A spec for a single-crate interface | `code/<crate>/README.md` or a sibling file |
| An ADR for a cross-cutting decision | `docs/ADR/` |
| An ADR for a single-crate or single-subsystem decision | `code/<crate>/README.md` or a sibling note |
| A crate-level README, module overview, or pattern note | `code/<crate>/README.md` or rustdoc |
| A choice of dependency, encoding, async runtime, error type, etc. | next to the code that uses it |
| An avionics-electronics decision (part choice, footprint rationale, layout note) | `hardware/README.md` or a sibling note |
| A mechanical / structural decision (material, print orientation, tolerances) | `structure/README.md` or a sibling note |
| A flight-simulation parameter choice or scenario rationale | `open rocket/README.md` or a sibling note |
| A pending implementation task | `docs/TODO.md` (system-wide) or the subsystem's own TODO/issues |
| A milestone plan or sequenced work breakdown | `docs/ROADMAP.md` |
| Vendor datasheets / external papers | `datasheets/` or `papers/` (do not paraphrase, link) |

## 6. Working conventions

- **Rust workspace** lives under `code/`. Run `cargo` commands from that directory unless a crate's README says otherwise. Use `cargo check`, `cargo clippy`, `cargo build`, and **`cargo nextest run`** for verifying code — these are pre-approved in the project's opencode config. See [`docs/toolchain.md`](docs/toolchain.md) for the full list of installed tools, targets, and toolchains.
  - **Never use `cargo test`.** Reaching for `cargo test` will produce misleading failures and waste the session debugging test infrastructure instead of real code. `cargo nextest run` isolates each test in its own process and eliminates these issues. 
- **Do not invent URLs or crate versions.** If you need a reference, point at `datasheets/`, `papers/`, or the upstream docs that are already cited.
- **Prefer editing existing docs** over creating new files. New files only when an existing doc would become incoherent.
- **Match the architecture/detailed-design split** when *reading*, too: if the user asks about a public interface, start in `docs/`; if they ask why a specific crate was chosen, start in that crate's README.

## 7. When this file is wrong

If something here contradicts what you observe in the repo, the repo wins — and this file is a bug. Fix it in the same change.

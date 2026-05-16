# AGENTS.md

Instructions for AI coding agents. Read this first — it's the entry point to all other docs.

## 1. Self-improving documentation

After every task, ask:

1. Did I learn something (goal, constraint, interface, non-obvious decision) not yet written down? → Update the relevant doc.
2. Did my change invalidate something written down? → Update the relevant doc.
3. Did I find a stale doc or one that contradicts the code? → Fix it, or note it in `docs/TODO.md`.

Treat documentation drift as a bug. Leave every doc better than you found it.

**Where to put new docs** — see §5. When unsure between two locations, pick one and add a one-line pointer from the other. Do not duplicate content.

## 2. Vocabulary

Full vocabulary: [`docs/GLOSSARY.md`](docs/GLOSSARY.md). Read it before writing prose.

Key distinctions:
- **Avionics** — the rocket subsystem (electronics + software, broadly)
  - **Flight computer** (a.k.a. **flight software**, **FC**) — software/firmware only
  - **Avionics electronics** — the PCB/schematic/parts (not the flight computer)
- **Rocket subsystem** ≠ **software subsystem** — different axes (see glossary)

## 3. Architecture vs. detailed design

Philosophy and Traceability Policy: [`docs/README.md`](docs/README.md). Read before classifying writing or authoring requirements.

**Folder mapping (quick reference):**

| Discipline | Architecture (`docs/`) | Detailed design |
|---|---|---|
| Software | FC ↔ ground-station interface; simulator ↔ FC interface; SITL/HITL goals | Crates, modules, serialization, task scheduling (`code/<crate>/`) |
| Avionics electronics | Buses, power domains, FC assumptions about the board | Schematic, PCB, parts, footprints (`hardware/`) |
| Mechanical | Airframe envelope, mass budget, recovery interface, mounting points | CAD, fillets, print orientation (`structure/`, `open rocket/`) |
| Systems | What the rocket must do; cross-discipline constraints | — |

**Workflow:**
- Before any architectural/interface change, a spec must exist or be updated.
- Before any significant design decision, write an ADR.
- Sketch architecture impact in `docs/` first; detailed design in the subsystem folder second.

**ADR rules:**
- **Do create** for non-obvious architectural decisions (e.g. choosing I2C vs SPI)
- **Do create** for non-obvious detailed-design decisions (e.g. choosing an async runtime from multiple contenders)
- **Don't create** for purely architectural tasks with no real decision (the spec is the right place)
- **Don't create** for mechanical detailed-design tasks where the decision is already made

Tasks in [`docs/ROADMAP.md`](docs/ROADMAP.md) follow the same split. Cross-subsystem contracts → `docs/software/spec.md`; implementation details → crate docs and `Cargo.toml`.

## 4. Project map

```
.
├── AGENTS.md              ← you are here
├── README.md              ← human-facing motivation + repo layout
│
├── docs/                  ← project-wide: goals, requirements, architecture, ADRs, progress, media
│   ├── README.md          ← architecture vs. detailed-design philosophy
│   ├── GLOSSARY.md        ← project vocabulary
│   └── toolchain.md       ← installed Rust tools & targets reference
│
├── code/                  ← Rust workspace (FC, ground station, proto, simulator, xtask)
│   └── README.md          ← crates overview + detailed-design index
│
├── hardware/              ← KiCad project, BOM, electronics design
├── open rocket/           ← OpenRocket simulation files
├── structure/             ← CAD models (FreeCAD/Fusion) for printable rocket parts
├── datasheets/            ← vendor datasheets (reference only)
├── papers/                ← research papers (reference only)
├── gps_config/            ← u-blox GPS configuration artifacts
└── .cargo/ .vscode/ .zed/ ← tooling configs
```

Each of `code/`, `hardware/`, `open rocket/`, `structure/` has a `README.md`. Cross-cutting concerns live in `docs/`. New top-level folders that aren't pure reference material need a `README.md` and a link here.

## 5. Where things go (cheat sheet)

| You want to write… | Put it in… |
|---|---|
| A new requirement (any discipline) | `docs/REQUIREMENTS.md` |
| A goal or non-goal for a subsystem | `docs/` (existing or new architecture doc) |
| A system-level or cross-subsystem interface contract / diagram | `docs/` |
| A spec for a cross-subsystem interface | `docs/software/` |
| A spec for a single-crate interface | `code/<crate>/README.md` or sibling file |
| An ADR for a cross-cutting decision | `docs/ADR/` |
| An ADR for a single-crate/subsystem decision | `code/<crate>/README.md` or sibling note |
| A crate-level README, module overview, or pattern note | `code/<crate>/README.md` or rustdoc |
| A dependency, encoding, runtime, or error-type choice | Next to the code that uses it |
| An avionics-electronics decision | `hardware/README.md` or sibling note |
| A mechanical/structural decision | `structure/README.md` or sibling note |
| A flight-simulation parameter or scenario rationale | `open rocket/README.md` or sibling note |
| A pending implementation task | `docs/TODO.md` or the subsystem's own TODO |
| A milestone plan or sequenced work breakdown | `docs/ROADMAP.md` |
| Vendor datasheets / external papers | `datasheets/` or `papers/` (do not paraphrase, link) |

## 6. Working conventions

- **Rust workspace** is under `code/`. Run `cargo` commands from there unless a crate's README says otherwise.
- **Approved verification commands:** `cargo check`, `cargo clippy`, `cargo build`, `cargo nextest run`
- **Never use `cargo test`** — it produces misleading failures; `cargo nextest run` isolates each test in its own process.
- **Do not invent URLs or crate versions.** Point at `datasheets/`, `papers/`, or already-cited upstream docs.
- **Config values that don't change at runtime use `pub const` on a unit struct,** not
  instance fields. See `flight-computer/src/config.rs` for the pattern — every config
  block is a unit struct with `pub const` associated items. Only use `Default` + fields
  when the config is genuinely loaded from somewhere (env, file, CLI args) at startup.
- **`no_std` crates get `std` in tests.** The `flight-computer` library enables `std` under
  `#[cfg(test)]`. Dev-dependencies and test code have full `std` access — do not annotate
  dev-deps as "no_std compatible" or qualify test-only imports with `no_std` constraints.
- **Prefer editing existing docs** over creating new files. New files only when an existing doc would become incoherent.
- **When reading:** start in `docs/` for public interfaces; start in the crate's README for implementation rationale.
- Full tool/target/toolchain list: [`docs/toolchain.md`](docs/toolchain.md)

## 7. When this file is wrong

If something here contradicts what you observe in the repo, the repo wins — and this file is a bug. Fix it in the same change.

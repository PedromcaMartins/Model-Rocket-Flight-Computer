# Motivation

This repository is the result of an ongoing project to develop a Model Rocket, while learning from many fields of engineering I find interesting.

My background is in Computer Science, and I've got 2 years of experience designing software for a rocket team @RED, Portugal. I want to challenge myself by using Rust, a language I've been learning since 2024, for the flight computer, and some of the Ground-station stack, if not all...

I also want to take this opportunity to learn more about model rocket design / simulation, get experience with CAD modeling by modeling and 3d printing parts for the rocket, electronics and PCB design, and eventually control algorithms :)

# Repository layout

```
.
├── code/            Rust workspace — flight computer library, ground-station backend,
│                    telemetry protocol, simulator, xtask. Embedded-target binary crates
│                    (cross-esp32-s3, cross-nucleo-f413zh) live alongside it.
├── hardware/        KiCad project for the avionics PCB, plus the electronics BOM.
├── structure/       CAD models (FreeCAD canonical, Fusion exports) for printable /
│                    machined airframe parts.
├── open rocket/     OpenRocket simulation files driving sizing, stability, and recovery
│                    decisions.
├── docs/            Project-wide documentation: goals, requirements, architecture, ADRs,
│                    glossary, progress.
├── datasheets/      Vendor datasheets.
├── papers/          Research papers.
├── gps_config/      u-blox GPS configuration artifacts.
└── .cargo/  .vscode/  .zed/    Tooling configuration.
```

# How this project is organized

This repo mixes software, electronics, mechanical, and systems engineering.

## Vocabulary

Project terms (rocket subsystems, *avionics*, *flight computer*, *avionics electronics*, the SITL/HITL distinction, …) are pinned in [`docs/GLOSSARY.md`](docs/GLOSSARY.md). 

## Architecture vs. detailed design

Documentation is split between:

- **Architecture & interface design** — *what* the system does and *why*. Goals, non-goals, constraints (with reasoning), and the public interfaces between subsystems. Lives in [`docs/`](docs/).
- **Detailed design** — *how* it is built. Choice of crate, schematic and layout, materials, file structure, build/fab/test recipes. Lives next to the artifact (`code/<crate>/`, `hardware/`, `structure/`, `open rocket/`).

The split applies to every discipline, not just code. See [`docs/README.md`](docs/README.md) for the full philosophy and rules of thumb.

## Tasks & maintenance

- **Requirements as a contract.** [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md) holds numbered, testable requirements (`[DEV-*]`, `[ROCKET-*]`, `[SW-*]`). New work should trace back to a requirement; if it doesn't, add one (with reasoning).
- **Decisions get ADRs.** Cross-cutting choices that affect more than one subsystem are recorded in [`docs/ADR/`](docs/ADR/) (see [`docs/README.md`](docs/README.md) for the ADR skeleton). Decisions internal to one crate or one PCB belong with that artifact.
- **Active TODOs.** [`docs/TODO.md`](docs/TODO.md) is the system-wide task list with checkboxes. Crate- or board-internal tasks live with the crate or board.
- **Progress tracked.** [`docs/ROADMAP.md`](docs/ROADMAP.md) tracks the multi-milestone plan to split the monolithic HOST binary into four processes. Its progress section is updated as milestones advance — tick a checkbox there when a task is done.
- **Per-revision folders.** `hardware/v1/`, `structure/v1/`, etc. freeze the moment that revision is fabricated or printed. Any incompatible change forks a new revision (`v2/`) — don't edit a frozen one.
- **Treat documentation drift as a bug.** If you change behavior and the docs don't match anymore, fix the docs in the same change.

## For AI agents

[`AGENTS.md`](AGENTS.md) contains agent-specific instructions (self-improvement protocol, project map, agent-flavored cheat sheet, working conventions). Humans contributing to the repo can ignore it.

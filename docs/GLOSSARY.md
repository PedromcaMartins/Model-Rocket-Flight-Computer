# Glossary

Terms used consistently across the project. If you write prose for this repo, this is the vocabulary to use.

## Systems & subsystems

**Rocket system** — the whole vehicle plus everything required to fly it. The top-level system this project builds.

**Rocket subsystems** — the engineering pieces of the rocket system:

| Rocket subsystem | Lives in |
|---|---|
| Avionics | `code/flight-computer/`, `hardware/`, and parts of `structure/` |
| Ground station | `code/ground-station-backend/` |
| Recovery | `structure/`, `code/flight-computer/` |
| Airframe / structure | `structure/`, `open rocket/` |
| Propulsion | — (commercial motor) |

When you write "system", say which one. "Rocket subsystem" and "software subsystem" (below) are not the same kind of thing.

## Avionics terminology

*Avionics* is the on-board electronics-and-software package as a whole. The term is intentionally broad — don't pin it to a fixed taxonomy. Two narrower terms inside it are pinned:

- **Flight computer** (a.k.a. **flight software**, abbreviated **FC**) — the *software* portion of avionics, i.e. the firmware. Lives in `code/flight-computer/` and the embedded crates that consume it. The PCB is **not** the flight computer.
- **Avionics electronics** — the *electronics* portion of avionics: PCB, schematic, parts, layout. Lives in `hardware/`.

When the distinction matters, say "flight computer" or "avionics electronics". When speaking about the package as a whole, "avionics" is the right word.

## Software systems & subsystems

A different axis from rocket subsystems. These are software-only artifacts; some map onto rocket subsystems, some are tooling that never flies.

| Software (sub)system | Maps to | Purpose |
|---|---|---|
| Flight computer (flight software) | Avionics | Runs on the rocket. |
| Ground-station backend / frontend | Ground station | Runs on the operator's machine. |
| Telemetry protocol (`proto`) | Cross-cutting | Wire-format contract. |
| Simulator | Tooling | Drives the FC under host execution. |
| SITL (Software-In-The-Loop) | Tooling | Simulator + FC, no hardware. |
| HITL (Hardware-In-The-Loop) | Tooling | Simulator + FC running on real hardware. |

# `open rocket/` — OpenRocket simulations

OpenRocket (`.ork`) project files used to evaluate flight profiles, sizing, stability, and recovery characteristics. Outputs from these simulations feed system-level requirements documented in [`../docs/REQUIREMENTS.md`](../docs/REQUIREMENTS.md) (see `[DEV-1]`).

## Files

| File | Purpose |
|---|---|
| `Pilot V1.ork` | Primary design candidate — the airframe currently being built and flown. |
| `Workshop Rocket.ork` | Workshop / scratch model used for experimenting with parameters. |

## Conventions

- One `.ork` per named design. When a design forks, bump its version (`Pilot V2.ork`) — do **not** overwrite the previous one.
- Simulation *results* that drive a requirement or design decision should be summarized in `../docs/` with a pointer back to the `.ork` they came from. Don't paraphrase the file; cite it.
- `.ork` files are zipped XML; small enough to track in git.

## See also

- [`../structure/`](../structure/) — CAD models for printable parts of the same airframe.
- [`../docs/REQUIREMENTS.md`](../docs/REQUIREMENTS.md) — `[ROCKET-*]` requirements that simulations validate.
- [`../AGENTS.md`](../AGENTS.md) — when to update this README.

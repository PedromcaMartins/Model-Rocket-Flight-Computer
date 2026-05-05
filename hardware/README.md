
# `hardware/` — Avionics electronics

KiCad project for the avionics PCB, plus the electronics BOM. This folder owns the *detailed* electrical design of the avionics — component selection rationale, footprint choices, layout decisions. Cross-cutting interface contracts (e.g. which buses are exposed to the flight computer) are architecture and live in [`../docs/`](../docs/). For vocabulary (avionics vs. flight computer vs. avionics electronics), see [`../docs/GLOSSARY.md`](../docs/GLOSSARY.md).

## Layout
```
hardware/
├── Electronics Parts.xlsx   ← BOM for the avionics electronics
└── v1/                      ← first PCB revision
    ├── v1.kicad_pro         (KiCad project)
    ├── v1.kicad_sch         (schematic)
    ├── v1.kicad_pcb         (PCB layout)
    ├── v1.kicad_prl         (per-user UI state — safe to ignore)
    └── v1-backups/          (KiCad auto-backups)
```

## Key components (v1)

| Role | Part |
|---|---|
| IMU | ICM-42688-P |
| Barometer | DPS368 |
| Magnetometer | MMC5983 |
| GNSS | u-blox NEO-M9N |
| LoRa radio | Ra-02 (SX1278) |

Datasheets are in [`../datasheets/`](../datasheets/). GPS configuration artifacts are in [`../gps_config/`](../gps_config/).

## Conventions

- **One subfolder per PCB revision** (`v1/`, `v2/`, …). When a design forks, bump its version — do **not** overwrite the previous one.
- Component-choice *reasoning* (why this IMU, why this radio band) belongs here as detailed design. The *requirement* it satisfies (e.g. `[SW-1A]`: must include IMU/baro/GPS) lives in [`../docs/REQUIREMENTS.md`](../docs/REQUIREMENTS.md).

## See also

- [`../structure/`](../structure/) — mechanical envelope this PCB has to fit.
- [`../code/`](../code/) — firmware that drives this hardware. The flight-computer crate is hardware-agnostic; per-target board bring-up lives in the embedded binary crates.

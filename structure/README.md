# `structure/` — Mechanical CAD

CAD models for the rocket's printable / machinable structural parts. Source files live here; manufactured outputs (STLs, drawings) are exported for reference and fabrication, but not edited in place. 

## Layout

```
structure/
├── Parts.xlsx       ← BOM for structural parts (off-the-shelf + printed)
└── v1/              ← first complete airframe revision
    ├── v1.FCStd            (FreeCAD project — primary source)
    ├── v1.<date>.FCBak     (FreeCAD auto-backup)
    └── Alpha v1 v2.f3d     (Fusion 360 export — original CAD)
```

## Conventions

<!--- **One subfolder per airframe revision** (`v1/`, `v2/`, …). Revisions ship together — if a single part changes incompatibly, it forces a new revision.-->
- **FreeCAD (`.FCStd`) is the canonical source.** Other formats (`.f3d`, `.step`, `.stl`) are exports.
- `Parts.xlsx` tracks the BOM for *structural* parts. Electronics BOM lives in [`../hardware/Electronics Parts.xlsx`](../hardware/).

## Design constraints driving this folder

These come from [`../docs/REQUIREMENTS.md`](../docs/REQUIREMENTS.md):

- `[ROCKET-5]` parts reusable.
- `[ROCKET-6]` strong but affordable.
- `[ROCKET-7]` 3D-printed where possible.
- `[ROCKET-8]` cardboard-tube fuselage.

Every structural decision must be traceable to a requirement in `docs/REQUIREMENTS.md`. A decision with no traceable requirement is incomplete — either identify the requirement it satisfies, or add a new one with its rationale and verification criteria (see [Traceability Policy](../docs/how-we-work.md#traceability-policy)).

## See also

- [`../open rocket/`](../open%20rocket/) — flight simulations that constrain mass, dimensions, and CG.
- [`../hardware/`](../hardware/) — electronics that this structure must house.

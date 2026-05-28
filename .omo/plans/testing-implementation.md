# Testing Implementation Plan

## Overview

Implement the three-tier testing strategy defined in `docs/testing-strategy.md`.
Build from the foundation up: unit tests first, then integration, then cross-crate,
then CI, then HW tests.

## Phases

### Phase 1 — Unit tests (current)
Add comprehensive `#[cfg(test)]` unit tests across all crates.

**Key files to create/modify:**
- `flight-computer/src/core/state_machine/` — FSM transition tests
- `flight-computer/src/core/state_machine/detectors/apogee_detector.rs` — apogee tests
- `flight-computer/src/core/state_machine/detectors/touchdown_detector.rs` — landing tests
- `flight-computer/src/config.rs` — config validation tests
- `flight-computer/src/core/storage.rs` — storage record tests
- `flight-computer/Cargo.toml` — add `hw_test` feature
- `proto/src/newtypes/` — property-based tests
- `simulator/src/physics/` — physics math tests
- `simulator/src/scripted/` — script engine tests
- All crate `Cargo.toml`s — extend clippy lints

**Order:** flight-computer (critical path) → proto → simulator → other crates.

### Phase 2 — Integration tests (next)
Add `code/<crate>/tests/` directories with behavior-focused tests.

### Phase 3 — Cross-crate tests (after P2)
Create `code/tests/` workspace member.

### Phase 4 — CI & infrastructure (after P1-P3)
GitHub Actions, coverage, xtask commands.

### Phase 5 — Embedded HW tests (ongoing alongside)
Test helpers in flight-computer, called from HW binary crates.

## Current phase: Phase 1
Starting with `flight-computer` unit tests (highest priority — FSM, detectors, config).

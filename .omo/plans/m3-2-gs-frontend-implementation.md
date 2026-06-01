# M3.2 — Ground Station Frontend: Implementation Plan

## TL;DR

> **Quick Summary**: Build the `ground-station-frontend` crate from scratch — a ratatui TUI binary with library/binary split that connects to the GS backend via WebSocket for live telemetry and REST for commands. Implements the complete M3.2 scope per the detailed spec.
>
> **Deliverables**:
> - `code/ground-station-frontend/spec.md` — detailed design spec
> - `code/ground-station-frontend/Cargo.toml` — crate manifest, workspace member
> - `code/ground-station-frontend/README.md` — crate overview
> - Library modules: `config.rs`, `history.rs`, `backend.rs`, `state.rs`, `lib.rs`
> - TUI binary modules: `main.rs`, `mod.rs`, `render.rs`, `telemetry.rs`, `logs.rs`, `controls.rs`
> - GS-FE-specific roadmap in `docs/ROADMAP.md`
>
> **Estimated Effort**: Large (14 implementation tasks + 4 review tasks)
> **Parallel Execution**: YES — 5 waves
> **Critical Path**: Task 1 (spec) → Task 2 (Cargo.toml) → Task 6 (backend) → Task 7 (state) → Task 8 (TUI core) → Task 13 (main.rs) → F1–F4

---

## Context

### Original Request
"Work through the plan — the spec looks to be complete, add it to ground-station-frontend crate. Afterwards make a series of tasks to incrementally implement and create a roadmap specific on the gs-fe crate with those tasks and milestones from docs/ROADMAP.md."

### Interview Summary
**Key Discussions**:
- The crate doesn't exist yet (empty dirs `src/`, `src/bin/`, `src/bin/tui/` only)
- Not registered as a workspace member in `code/Cargo.toml`
- The spec in `.omo/drafts/m3-2-gs-frontend-spec-complete.md` is the authoritative detailed design
- Library/binary split: library owns transport + state + pollers; binary owns UI
- WS streaming protocol (not REST polling): connects to `ws://127.0.0.1:8000/api/records`
- Three-tab TUI: Telemetry (raw values + recent history), Logs (placeholder), Controls (arm/ignite/reconnect)
- Disconnect UX: red banner, dimmed stale data, no auto-retry, reconnect on `r`
- Heartbeat via ping REST endpoint with latency display
- M3.4 (3D rendering) deferred — not in this plan

**Research Findings**:
- Spec pattern: `simulator/spec.md` is the canonical crate-level spec
- Config pattern: unit struct with `pub const` (see `ground-station-backend/src/config.rs`)
- TUI lifecycle: crossterm raw mode, alternate screen, `spawn_blocking` render loop (simulator pattern)
- Error handling: `anyhow::Result`, `.context()`, no `.expect()` in production
- Edition: `2024` across workspace
- Proto features needed: `"client"` (for `proto::record::Record` types)
- Backend needs `rocket_ws` or equivalent to serve the WS endpoint — flagged as cross-crate dependency

### Oracle Phase 1 Verification
> **CHECK 5/5 PASS | VERDICT: GO**
> Phase 1 interview is complete. Scope IN/OUT explicit. Test strategy settled (none + agent QA). No contradictions with codebase patterns.

---

## Work Objectives

### Core Objective
Build the `ground-station-frontend` crate implementing the M3.2 detailed design spec — a WebSocket-connected ratatui TUI providing live telemetry, logs, and controls for the ground station operator.

### Concrete Deliverables
- `code/ground-station-frontend/spec.md` — detailed design document
- `code/ground-station-frontend/Cargo.toml` — crate manifest
- `code/ground-station-frontend/README.md` — crate overview
- Library crate at `code/ground-station-frontend/src/` with `config.rs`, `history.rs`, `backend.rs`, `state.rs`, `lib.rs`
- TUI binary crate at `code/ground-station-frontend/src/bin/tui/` with `main.rs`, `mod.rs`, `render.rs`, `telemetry.rs`, `logs.rs`, `controls.rs`
- GS-FE sub-milestones added to `docs/ROADMAP.md` §M3.2
- `code/Cargo.toml` workspace members updated with `"ground-station-frontend"`
- WS broadcast endpoint at `/api/records` added to `ground-station-backend`
- REST command routes `POST /api/commands/arm` and `POST /api/commands/ignite` added to `ground-station-backend`

### Definition of Done
- [ ] `cargo check -p ground-station-frontend` passes (no errors)
- [ ] `cargo clippy -p ground-station-frontend` passes (no warnings)
- [ ] `cargo build -p ground-station-frontend` succeeds
- [ ] Binary starts and renders TUI layout (manual verification with backend running)

### Must Have
- WS client receiving telemetry `Record`s and status updates from backend
- REST client issuing arm/ignite/ping commands
- Three-tab TUI: Telemetry, Logs (placeholder), Controls
- Disconnect UX: dimmed stale data, red status, reconnect button
- Heartbeat with latency display (blinking ● indicator)
- Library/binary split — library owns transport, state, pollers

### Must NOT Have (Guardrails)
- No postcard-rpc in the frontend (architectural constraint — all data through GS backend)
- No M3.4 3D rendering (ratatui-ratty deferred)
- No velocity/acceleration derivation from altitude deltas (FC is source of truth)
- No Braille time-series charts (superseded by future 3D view)
- No auto-reconnect — operator must press `r`
- No `unwrap()` or `expect()` in production code paths
- No `unsafe` blocks

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (`bun test` via `cargo nextest run`)
- **Automated tests**: None (for this crate — follows ground-station-backend pattern)
- **Verification method**: Agent-executed QA scenarios + `cargo check` + `cargo clippy` + `cargo build`

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to `.omo/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Library build**: Use `bash` — `cargo check -p ground-station-frontend`, `cargo clippy -p ground-station-frontend`
- **Binary startup**: Use `interactive_bash` (tmux) — run binary, verify TUI renders, send keystrokes, capture terminal output
- **Integration**: Start full HOST stack (FC + Sim + GS backend + GS frontend), verify telemetry flows

---

## GS-FE Sub-Milestone Roadmap

The GS-FE implementation is organized into 4 sub-milestones within M3.2:

| Sub-MS | Name | Tasks | Status |
|---|---|---|---|---|
| M3.2a | Foundation + WS Backend | 0–4: WS endpoint, spec, Cargo.toml, config, history | Planned |
| M3.2b | Core Library | 5–7: backend client, state, lib.rs | Planned |
| M3.2c | TUI Infrastructure | 8–12: terminal, render, telemetry, logs, controls | Planned |
| M3.2d | Binary Entry + Polish | 13–14: main.rs, README | Planned |

> **Note:** Task 0 (WS endpoint) modifies the `ground-station-backend` crate — it's a cross-crate pre-requisite.
> Task 1 (spec.md) will be written by the executing agent — Prometheus cannot write outside `.omo/`. The draft at `.omo/drafts/m3-2-gs-frontend-spec-complete.md` is the source.

These sub-milestones are tracked in the GS-FE progress section added to `docs/ROADMAP.md`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 0 (Backend pre-requisite — must complete first):
└── Task 0: Add WS broadcast endpoint to GS backend [unspecified-high]

Wave 1 (Foundation — start after Wave 0, max parallel):
├── Task 1: Write spec.md to crate [writing]
├── Task 2: Create Cargo.toml + workspace registration [quick]
├── Task 3: Create config.rs (pub const unit struct) [quick]
└── Task 4: Create history.rs (RollingHistory<T>) [quick]

Wave 2 (Core library — depends on Wave 1, max parallel):
├── Task 5: Create backend.rs (BackendClient + WsBackend + RestClient) [unspecified-high]
├── Task 6: Create state.rs (AppState + run_ws_reader) [unspecified-high]
└── Task 7: Create lib.rs (module decls + re-exports) [quick]

Wave 3 (TUI core — depends on Wave 2):
├── Task 8: Create tui/mod.rs (terminal setup + event loop) [unspecified-high]
└── Task 9: Create render.rs (layout dispatch by active tab) [quick]

Wave 4 (TUI tabs — depends on Wave 3, max parallel):
├── Task 10: Create telemetry.rs (Tab 1: raw values + recent history) [unspecified-high]
├── Task 11: Create logs.rs (Tab 2: log tail placeholder) [quick]
└── Task 12: Create controls.rs (Tab 3: arm, ignite, reconnect) [unspecified-high]

Wave 5 (Binary entry + polish — depends on Wave 4):
├── Task 13: Create main.rs (entry point + WS reader spawn) [unspecified-high]
└── Task 14: Create README.md + update ROADMAP.md [writing]

Wave FINAL (After ALL tasks — 4 parallel reviewers):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 0 → Task 2 → Task 5 → Task 6 → Task 8 → Task 10 → Task 13 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 4 (Waves 1 & 4)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|---|---|---|
| 0 (WS endpoint) | — | 5, 6, 13, F3 |
| 1 (spec) | — | — |
| 2 (Cargo.toml) | — | 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13 |
| 3 (config.rs) | 2 | 5, 6, 8, 9, 10, 11, 12, 13 |
| 4 (history.rs) | 2 | 6, 10 |
| 5 (backend.rs) | 0, 2, 3 | 6, 7, 8, 13 |
| 6 (state.rs) | 0, 2, 3, 4, 5 | 8, 13 |
| 7 (lib.rs) | 2, 5 | 8, 9, 10, 11, 12, 13 |
| 8 (tui/mod.rs) | 2, 3, 5, 6, 7 | 9, 10, 11, 12, 13 |
| 9 (render.rs) | 2, 3, 8 | 10, 11, 12, 13 |
| 10 (telemetry.rs) | 2, 3, 4, 6, 7, 8, 9 | 13 |
| 11 (logs.rs) | 2, 3, 6, 7, 8, 9 | 13 |
| 12 (controls.rs) | 2, 3, 6, 7, 8, 9 | 13 |
| 13 (main.rs) | all above | F1-F4 |
| 14 (README) | 2 | — |

---

## TODOs

### Wave 0 — Backend Pre-requisite

- [ ] 0. **Add WS broadcast endpoint + REST command routes to GS backend**

  **What to do**:
  - **Part A — WS broadcast**:
    - `rocket_ws` is Rocket v0.5's **official, native** WebSocket crate (built on Rocket's HTTP upgrade API, same server, same port). Add `rocket_ws = "0.1.1"` (latest compatible with Rocket 0.5.1) to `code/ground-station-backend/Cargo.toml`. **Ask the user if uncertain about the version.**
    - Create a tokio broadcast channel (`tokio::sync::broadcast::channel(256)`) as shared state in the GS backend
    - Pass the broadcast sender into the FC client loop so each received `Record` is broadcast to all WS clients as `{"type":"record","data":{...}}` JSON
    - Add a WS route at `/records` in the GS backend's Rocket app (under `/api` mount):
      - On connect: send a status snapshot with the latest record included: `{"type":"status","data":{"connected":true,"session_start":"...","record_count":N,"latest_record":{...}}}`
      - On new record broadcast: forward to all connected WS clients
      - On disconnect: clean up
    - Wrap the broadcast sender in `rocket::State<>` for route access
    - Use `rocket_ws::Stream!` for the WS handler
  - **Part B — REST command routes**:
    - Add `POST /api/commands/arm` route following the same pattern as the existing `POST /api/commands/ping`:
      - Grab the `FcConnection` client from `AppState`
      - Call the postcard-rpc arm endpoint on the FC
      - Return `200` with success or `503` with error
    - Add `POST /api/commands/ignite` route, same pattern
    - Register both new routes in `main.rs` alongside existing routes
  - **Part C — Add ?limit param + remove redundant GET routes**:
    - Add optional `?limit=N` query parameter to `GET /api/records` — returns only the last N records when specified (e.g., `GET /api/records?limit=1000`). The frontend uses this for bootstrap: fetch recent records before opening WS.
    - Remove `GET /api/records/latest` route (`records_latest`) from `routes.rs` — WS streaming provides latest to the frontend
    - Remove `GET /api/logs` route (`logs`) from `routes.rs` — log forwarding deferred to M3.6
    - Remove both from the `rocket::routes![]` macro in `main.rs`
    - Keep `GET /api/records` — useful for debugging/inspection + frontend bootstrap

  **Must NOT do**:
  - Do not remove `GET /api/records` (kept for debugging)
  - Do not change the existing `GET /api/status` or `POST /api/commands/ping` routes
  - Do not add sim-gs.sock integration (M3.3)
  - Do not add log message forwarding (M3.6)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed
  - **Reason**: Requires understanding Rocket routing, WS lifecycle, broadcast channels, and integration with existing FC client loop

  **Parallelization**:
  - **Can Run In Parallel**: NO (pre-requisite for integration testing)
  - **Parallel Group**: Wave 0 (must complete before full integration)
  - **Blocks**: Tasks 5, 6, 13 (backend types, state reader, full binary)
  - **Blocked By**: None

  **References**:
  - `code/ground-station-backend/Cargo.toml` — add rocket_ws dep
  - `code/ground-station-backend/src/main.rs` — Rocket app build, routes, register new routes
  - `code/ground-station-backend/src/routes.rs` — existing ping handler (pattern for arm/ignite), AppState
  - `code/ground-station-backend/src/fc_client.rs` — FC client loop, FcConnection, broadcast integration
  - `rocket_ws` docs: `https://docs.rs/rocket_ws/latest/rocket_ws/` — WS route handler API
  - Spec §2 (WS protocol contract — message format frontend expects)
  - `tokio::sync::broadcast` docs — channel API

  **Acceptance Criteria**:
  - [ ] `rocket_ws` added to `ground-station-backend/Cargo.toml`
  - [ ] Broadcast channel created in `AppState` / shared via Rocket `manage()`
  - [ ] FC client loop publishes Records to broadcast channel after writing to storage
  - [ ] WS route at `/records` (under `/api` mount) subscribes to broadcast and pushes JSON to clients
  - [ ] WS status message on connect includes `latest_record` field for immediate frontend state
  - [ ] `POST /api/commands/arm` route exists and follows same pattern as existing ping
  - [ ] `POST /api/commands/ignite` route exists and follows same pattern
  - [ ] `GET /api/records/latest` route removed from `routes.rs` and `main.rs`
  - [ ] `GET /api/logs` route removed from `routes.rs` and `main.rs`
  - [ ] `GET /api/records` still present (kept for debugging)
  - [ ] `cargo check -p ground-station-backend` passes
  - [ ] `cargo build -p ground-station-backend` succeeds

  **QA Scenarios**:
  ```
  Scenario: Backend compiles with WS + new routes
    Tool: Bash
    Preconditions: None
    Steps:
      1. cargo check -p ground-station-backend 2>&1 | tail -5
      2. cargo build -p ground-station-backend 2>&1 | tail -5
    Expected Result: Both commands return "Finished" — no errors
    Evidence: .omo/evidence/task-0-backend-ws-build.txt

  Scenario: WS endpoint is reachable
    Tool: Bash
    Preconditions: Backend running with FC connected
    Steps:
      1. timeout 3 bash -c "echo '' | websocat ws://127.0.0.1:8000/api/records 2>&1 || true"
    Expected Result: WebSocket connection established (no connection refused)
    Evidence: .omo/evidence/task-0-backend-ws-connect.txt
  ```

  **Commit**: YES (standalone pre-requisite)
  - Message: `feat(gs-be): add WS broadcast, arm/ignite routes; remove records/latest, logs endpoints`
  - Files: `code/ground-station-backend/Cargo.toml`, `code/ground-station-backend/src/main.rs`, `code/ground-station-backend/src/routes.rs`, `code/ground-station-backend/src/fc_client.rs`

### Wave 1 — Foundation

- [ ] 1. **Write spec.md to crate**

  **What to do**:
  - Copy the contents of `.omo/drafts/m3-2-gs-frontend-spec-complete.md` to `code/ground-station-frontend/spec.md`
  - Fix typos: `Duratoin` → `Duration`, `Duratiton` → `Duration`, `from_seconds` → `from_secs`
  - The spec is the authoritative detailed design for this crate. All subsequent tasks must follow it.

  **Must NOT do**:
  - Do not change the spec content beyond fixing the typos
  - Do not add M3.4 (3D rendering) content — it's spec'd for future work only

  **Recommended Agent Profile**:
  - **Category**: `writing`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Nothing directly (informational)
  - **Blocked By**: None

  **References**:
  - `.omo/drafts/m3-2-gs-frontend-spec-complete.md` — source content to copy
  - `code/simulator/spec.md` — existing spec pattern to match for file conventions

  **Acceptance Criteria**:
  - [ ] `code/ground-station-frontend/spec.md` exists
  - [ ] `Duration` is spelled correctly (not `Duratoin` or `Duratiton`)
  - [ ] File starts with `# ground-station-frontend — detailed design`
  - [ ] All sections 1–11 present and match the source draft

  **QA Scenarios**:
  ```
  Scenario: Spec file exists and is well-formed
    Tool: Bash (cat + grep)
    Preconditions: None
    Steps:
      1. cat code/ground-station-frontend/spec.md | head -1
      2. grep "## 11. Verification" code/ground-station-frontend/spec.md
    Expected Result: (1) Shows "# ground-station-frontend — detailed design"
                        (2) Shows "## 11. Verification"
    Evidence: .omo/evidence/task-1-spec-exists.txt
  ```

  **Commit**: YES
  - Message: `feat(gs-fe): add detailed design spec for M3.2`
  - Files: `code/ground-station-frontend/spec.md`

- [ ] 2. **Create Cargo.toml + register workspace member**

  **What to do**:
  - Create `code/ground-station-frontend/Cargo.toml` with dependencies per spec §8
  - Dependency versions in the spec are **suggestions** — use latest compatible versions unless there's a known compatibility issue. Prefer `tokio = "1"` (no pin — `=1.49` is too restrictive) and `ratatui = "0.29"`, `crossterm = "0.28"` as floor versions. **Ask the user if unsure about any version.**
  - Add `"ground-station-frontend"` to `members = [...]` in `code/Cargo.toml`
  - Binary name: `ground-station-frontend`, path: `src/bin/tui/main.rs`
  - Use edition `"2024"` to match workspace convention
  - Proto features: `["client"]` (do not add `transport-ipc` — frontend never speaks postcard-rpc)
  - Reqwest: use `rustls-tls` feature, not `native-tls` (matches workspace convention)

  **Must NOT do**:
  - Do not add `transport-ipc` proto feature — frontend never speaks postcard-rpc
  - Do not add `ratatui-ratty` — that's M3.4
  - Do not use `native-tls` for reqwest

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3, 4)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3–14 (all subsequent tasks)
  - **Blocked By**: None

  **References**:
  - `code/ground-station-backend/Cargo.toml` — existing crate manifest pattern (edition, proto dep style)
  - `code/Cargo.toml` — workspace root, add to members list
  - Spec §8 (cargo dependencies) and §3 (crate structure)

  **Acceptance Criteria**:
  - [ ] `code/ground-station-frontend/Cargo.toml` exists and is valid TOML
  - [ ] Cargo.toml lists all deps from spec §8: tokio-tungstenite, futures-util, reqwest, ratatui, crossterm, tokio, tokio-util, arc-swap, async-trait, anyhow, serde, serde_json, chrono, tracing, scopeguard
  - [ ] `"ground-station-frontend"` is in `code/Cargo.toml` workspace members
  - [ ] `cargo check` succeeds (may have unused imports — will be fixed by later tasks)

  **QA Scenarios**:
  ```
  Scenario: Cargo.toml is valid and workspace member registered
    Tool: Bash
    Preconditions: None
    Steps:
      1. cat code/ground-station-frontend/Cargo.toml
      2. grep "ground-station-frontend" code/Cargo.toml
      3. cargo metadata --format-version 1 | python -c "import sys,json; d=json.load(sys.stdin); print([p['name'] for p in d['packages'] if 'frontend' in p['name']])"
    Expected Result: (1) Valid TOML with all required fields
                        (2) Shows "ground-station-frontend" in members list
                        (3) Returns ["ground-station-frontend"]
    Evidence: .omo/evidence/task-2-cargo-metadata.txt
  ```

  **Commit**: YES (groups with Task 1)
  - Message: `feat(gs-fe): add foundation — spec, cargo.toml, config, history`
  - Files: `code/ground-station-frontend/Cargo.toml`, `code/Cargo.toml`

- [ ] 3. **Create config.rs (pub const unit struct)**

  **What to do**:
  - Create `code/ground-station-frontend/src/config.rs`
  - Follow the unit struct pattern with `pub const` values (per AGENTS.md §6)
  - Implement per the spec §7, but make these values `pub const` so they're configurable (not hardcoded):
    - `BACKEND_HOST`, `BACKEND_PORT`: mirror the backend's `REST_HOST`/`REST_PORT` values (127.0.0.1, port 8000)
    - `WS_PATH`: `/api/records`
    - `TUI_FPS`: 60
    - `HISTORY_WINDOW`: `Duration::from_secs(30)`
    - `HISTORY_MAX_SAMPLES`: 1000 (changed from draft's 500)
    - `LOG_BUFFER_CAPACITY`: 1000 (changed from draft's 2000)
    - `LOG_VISIBLE_LINES`: 20
    - `PING_INTERVAL`: `Duration::from_millis(1000)` (configurable pub const)
  - URL helper constants (all `const &str` — compile-time zero-cost, not functions):
    - `pub const WS_URL: &str = "ws://127.0.0.1:8000/api/records";`
    - `pub const RECORDS_URL: &str = "http://127.0.0.1:8000/api/records";`
    - `pub const ARM_URL: &str = "http://127.0.0.1:8000/api/commands/arm";`
    - `pub const IGNITE_URL: &str = "http://127.0.0.1:8000/api/commands/ignite";`
    - `pub const PING_URL: &str = "http://127.0.0.1:8000/api/commands/ping";`
  - Fix the typos from the draft (`Duratoin` → `Duration`, `Duratiton` → `Duration`, `from_seconds` → `from_secs`)

  **Must NOT do**:
  - Do not make Config an instantiatable struct — must be unit struct with `pub const`
  - Do not add M3.4 constants

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2, 4)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 5, 6, 8, 9, 10, 11, 12, 13
  - **Blocked By**: Task 2 (Cargo.toml must exist first)

  **References**:
  - `code/ground-station-backend/src/config.rs` — canonical config unit struct pattern
  - Spec §7 (config)
  - `AGENTS.md §6` — config pattern rules

  **Acceptance Criteria**:
  - [ ] `src/config.rs` exists with `pub struct Config;` and its impl block
  - [ ] All `Duration` values use correct Rust API (`Duration::from_secs`, `Duration::from_millis`)
  - [ ] `cargo check -p ground-station-frontend` passes (after Cargo.toml exists)

  **QA Scenarios**:
  ```
  Scenario: Config compiles and URL helpers are correct
    Tool: Bash
    Preconditions: Cargo.toml and workspace member exist
    Steps:
      1. cargo check -p ground-station-frontend 2>&1 | tail -5
      2. cargo doc -p ground-station-frontend --no-deps 2>&1 | tail -3
    Expected Result: (1) "Checking ground-station-frontend v0.0.0 ... Finished"
                        (2) Documentation builds without errors
    Evidence: .omo/evidence/task-3-config-check.txt

  Scenario: No Duration typos
    Tool: Bash (grep)
    Preconditions: config.rs exists
    Steps:
      1. grep -n "Duratoin\|Duratiton\|from_seconds" src/config.rs || echo "CLEAN"
    Expected Result: (1) "CLEAN" — no typos found
    Evidence: .omo/evidence/task-3-config-typos-check.txt
  ```

  **Commit**: YES (groups with Tasks 1, 2, 4)

- [ ] 4. **Create history.rs (RollingHistory\<T\>)**

  **What to do**:
  - Create `code/ground-station-frontend/src/history.rs`
  - Implement `RollingHistory<T>` generic ring buffer per spec §4.3
  - Fields: `samples: VecDeque<(Instant, T)>`, `window: Duration`, `max_len: usize`
  - Methods: `new(window, max_len)`, `push(time, value)`, `latest()`, `snapshot()`, `len()`
  - `push()` evicts samples older than `window` on each call
  - Bound `T: Clone` for `snapshot()` and `latest()`
  - Use `std::time::Instant` (host crate, no embassy)
  - The `max_len` and `window` parameters come from `Config` — consumers pass them in from `Config::HISTORY_MAX_SAMPLES` (default: 1000) and `Config::HISTORY_WINDOW` (default: 30s)

  **Must NOT do**:
  - Do not hardcode capacity inside the struct — accept as constructor parameter
  - Do not make it specific to altitude — generic over `T`
  - Do not add velocity/acceleration derivation (FC is source of truth)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2, 3)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 6, 10
  - **Blocked By**: Task 2 (Cargo.toml must exist first)

  **References**:
  - Spec §4.3 (RollingHistory API)
  - `std::collections::VecDeque` — backing storage
  - `std::time::Instant` — time type

  **Acceptance Criteria**:
  - [ ] `src/history.rs` exists
  - [ ] `RollingHistory<T>` struct with `new()`, `push()`, `latest()`, `snapshot()`, `len()`
  - [ ] `T: Clone` bound on methods that return data
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: RollingHistory compiles and basic push/snapshot works (unit test)
    Tool: Bash
    Preconditions: Cargo.toml exists
    Steps:
      1. Add a #[cfg(test)] module in history.rs with:
         - Test: push 3 f32 values, verify snapshot() returns 3
         - Test: push values older than window, verify they are evicted
         - Test: latest() returns most recent value
         - Test: push more than max_len values, verify only max_len retained
      2. cargo test -p ground-station-frontend -- history 2>&1 | tail -10
    Expected Result: All 4 history tests pass
    Evidence: .omo/evidence/task-4-history-tests.txt
  ```

  > **Tests optional**: the unit tests above are suggested but not mandatory. If you want to skip them, that's fine — agent QA scenarios are sufficient.

  **Commit**: YES (groups with Tasks 1, 2, 3)

### Wave 2 — Core Library

- [ ] 5. **Create backend.rs (BackendClient + WsBackend + RestClient)**

  **What to do**:
  - Create `code/ground-station-frontend/src/backend.rs`
  - Implement per spec §4.1:
    - `BackendClient` trait (async_trait, Send + Sync)
    - `WsMessageStream` trait (async_trait, Send)
    - `WsMessage` enum: Record(Record), Log(String), Status{connected, session_start, record_count}
    - `WsBackend` struct implementing BackendClient — wraps reqwest Client for REST, tokio-tungstenite for WS
  - `connect_ws()`: open WS to `Config::ws_url()`, return `Box<dyn WsMessageStream>`
  - WS message parsing: read JSON text frames, dispatch on `"type"` discriminator
    - `type: "record"` → deserialize `data` as `proto::record::Record`
    - `type: "log"` → extract `data` as String
    - `type: "status"` → deserialize `data` as `{connected, session_start, record_count}`
  - `arm()`: POST to `Config::arm_url()`
  - `ignite()`: POST to `Config::ignite_url()`
  - `ping()`: POST to `Config::ping_url()`, parse latency from response
  - **Bootstrap method** — `fetch_recent_records(limit: usize) -> anyhow::Result<Vec<proto::record::Record>>`:
    - GET `Config::records_url()?limit={limit}`
    - Returns the last N records for bootstrapping RollingHistory before WS connect
    - Called once at startup in `main.rs` before spawning the WS reader

  **Must NOT do**:
  - Do not add postcard-rpc types or imports
  - Do not add transport-ipc feature (enforced by Cargo.toml)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed
  - **Reason**: Medium complexity — WS client + REST client + JSON deserialization

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7)
  - **Blocks**: Tasks 6, 7, 8, 13
  - **Blocked By**: Tasks 2, 3

  **References**:
  - Spec §2 (WS protocol contract) — message format and types
  - Spec §4.1 (BackendClient API)
  - `proto/src/record.rs` — `Record` type for deserialization
  - `tokio-tungstenite` docs: `connect_async`, `Message::Text`
  - `reqwest` docs: `Client::post`, `.json()`, `Client::get`

  **Acceptance Criteria**:
  - [ ] `src/backend.rs` exists with all trait definitions and implementations
  - [ ] `BackendClient` trait with connect_ws, arm, ignite, ping, fetch_recent_records
  - [ ] `WsMessage` enum with all three variants
  - [ ] `WsBackend` struct implementing `BackendClient`
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: Backend module compiles without warnings
    Tool: Bash
    Preconditions: config.rs and Cargo.toml exist
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Checking ground-station-frontend ... Finished" — no errors
    Evidence: .omo/evidence/task-5-backend-check.txt
  ```

  **Commit**: YES (groups with Tasks 6, 7)
  - Message: `feat(gs-fe): add core library — backend client, state, lib.rs`

- [ ] 6. **Create state.rs (AppState + run_ws_reader)**

  **What to do**:
  - Create `code/ground-station-frontend/src/state.rs`
  - Implement per spec §4.2:
    - `AppState` struct with atomic fields, Mutex fields, ArcSwap field
    - `TransitionEvent` struct
    - `run_ws_reader()` async function: opens WS, dispatches messages
  - Concurrency model per spec:
    - `AtomicBool` for `connected`, `ping_heartbeat`
    - `AtomicU64` for `record_count`, `latency_ms` (fixed-point hundredths)
    - `Mutex<String>` for `session_start`, `last_error`
    - `ArcSwap<Option<Record>>` for `latest_record`
    - `Mutex<VecDeque<String>>` for `log_buffer`
    - `Mutex<Vec<TransitionEvent>>` for `transitions`
    - `Mutex<RollingHistory<f32>>` for `altitude_history`
    - `Mutex<RollingHistory<GpsCoordinates>>` for `gps_history`
  - WS message dispatch per spec §4.4:
    - `record`: update ArcSwap, push to history, check FlightState for transitions
    - `log`: no-op in M3.2 (handler wired for forward compat)
    - `status`: update connected, session_start, record_count
  - `log_buffer`: use `VecDeque::with_capacity(Config::LOG_BUFFER_CAPACITY)` and cap at that capacity on push (default: 1000)
  - `altitude_history` / `gps_history`: initialize with `Config::HISTORY_WINDOW` and `Config::HISTORY_MAX_SAMPLES` (default: 30s window, 1000 samples)

  **Must NOT do**:
  - Do not add auto-retry logic — WS reader exits on disconnect
  - Do not add velocity/acceleration derivation
  - Do not add M3.6 log message handling (no-op is correct for M3.2)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7)
  - **Blocks**: Tasks 8, 13
  - **Blocked By**: Tasks 2, 3, 4, 5

  **References**:
  - Spec §4.2 (AppState + concurrency model)
  - Spec §4.4 (WS reader task)
  - `arc-swap` docs: `ArcSwap::load()`, `ArcSwap::store()`, `ArcSwap::rcu()`
  - `tokio_util::sync::CancellationToken` — cancellation pattern
  - `code/ground-station-backend/src/` — existing GS concurrency patterns

  **Acceptance Criteria**:
  - [ ] `src/state.rs` exists
  - [ ] `AppState` has all fields from spec §4.2
  - [ ] `TransitionEvent` struct defined
  - [ ] `run_ws_reader()` async function exists
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: State module compiles
    Tool: Bash
    Preconditions: All Wave 1 tasks and Task 5 done
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Checking ground-station-frontend ... Finished" — no errors
    Evidence: .omo/evidence/task-6-state-check.txt
  ```

  **Commit**: YES (groups with Tasks 5, 7)

- [ ] 7. **Create lib.rs (module decls + re-exports)**

  **What to do**:
  - Create `code/ground-station-frontend/src/lib.rs`
  - Declare public modules: `pub mod config;`, `pub mod history;`, `pub mod backend;`, `pub mod state;`
  - Re-export key types for convenience:
    - `pub use config::Config;`
    - `pub use history::RollingHistory;`
    - `pub use backend::{BackendClient, WsBackend, WsMessageStream, WsMessage};`
    - `pub use state::{AppState, TransitionEvent, run_ws_reader};`

  **Must NOT do**:
  - Do not add binary-specific re-exports (TUI modules are private to the binary)
  - Do not add any implementation logic

  **Recommended Agent Profile**:
  - **Category**: `quick`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6)
  - **Blocks**: Tasks 8, 9, 10, 11, 12, 13
  - **Blocked By**: Tasks 2, 5

  **References**:
  - Spec §3 (crate structure)
  - `code/simulator/src/lib.rs` — existing library crate pattern
  - `code/ground-station-backend/src/main.rs` — binary-only crate (lib.rs not needed there)

  **Acceptance Criteria**:
  - [ ] `src/lib.rs` exists
  - [ ] Public modules match spec §3: config, history, backend, state
  - [ ] Key types re-exported
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: Library compiles and exports correct types
    Tool: Bash
    Preconditions: All Wave 1-2 tasks done
    Steps:
      1. cargo check -p ground-station-frontend 2>&1 | tail -3
      2. grep "pub mod\|pub use" src/lib.rs
    Expected Result: (1) "Finished" — no errors
                        (2) Shows all 4 modules and re-exports
    Evidence: .omo/evidence/task-7-lib-check.txt
  ```

  **Commit**: YES (groups with Tasks 5, 6)

### Wave 3 — TUI Core

- [ ] 8. **Create tui/mod.rs (terminal setup + event loop)**

  **What to do**:
  - Create `code/ground-station-frontend/src/bin/tui/mod.rs`
  - Implement terminal lifecycle:
    - crossterm raw mode + alternate screen
    - scopeguard panic guard to restore terminal on panic
    - Event loop: render at TUI_FPS, handle keyboard events
  - `ActiveTab` enum: Telemetry, Logs, Controls
  - Keyboard dispatch:
    - `Tab` / `Shift+Tab` or `1`, `2`, `3`: switch tabs
    - `q` / `Ctrl+C`: quit
    - `a`: arm (spawn tokio task → calls `backend.arm()`)
    - `i`: ignite (spawn tokio task → calls `backend.ignite()`)
    - `r`: reconnect (spawn new WS reader task)
  - Expose `run_tui(state: Arc<AppState>) -> anyhow::Result<()>`

  **Must NOT do**:
  - Do not add M3.5 mouse support
  - Do not add M3.4 3D rendering

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Wave 2)
  - **Parallel Group**: Sequential within Wave 3
  - **Blocks**: Tasks 9, 10, 11, 12, 13
  - **Blocked By**: Tasks 2, 3, 5, 6, 7

  **References**:
  - Spec §5 (TUI layout, status bar)
  - Spec §6 (disconnect UX)
  - `code/simulator/src/bin/host/tui/` — simulator TUI setup (crossterm raw mode, alternate screen, event loop)
  - `crossterm` docs: `event::read()`, `execute!`, `terminal::enable_raw_mode()`, `AlternateScreen`
  - `ratatui` docs: `DefaultTerminal`, `Terminal::draw()`

  **Acceptance Criteria**:
  - [ ] `src/bin/tui/mod.rs` exists
  - [ ] `run_tui(state: Arc<AppState>)` function
  - [ ] Keyboard dispatch handles Tab, q, a, i, r
  - [ ] scopeguard defer! for terminal restoration
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: TUI module compiles
    Tool: Bash
    Preconditions: All Wave 1-2 tasks done
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Finished" — no errors
    Evidence: .omo/evidence/task-8-tui-mod-check.txt
  ```

  **Commit**: YES (groups with Task 9)
  - Message: `feat(gs-fe): add TUI core — terminal setup, render dispatch`

- [ ] 9. **Create render.rs (layout dispatch by active tab)**

  **What to do**:
  - Create `code/ground-station-frontend/src/bin/tui/render.rs`
  - Implement the top-level layout rendering function
  - Status bar (row 1, always visible): connected status ●/○, latency, session info, tab switcher
  - Active tab dispatch: calls the appropriate tab renderer
  - Disconnect overlay: red banner when `connected == false`
  - Uses ratatui `Layout` with vertical split for status bar + content area

  **Must NOT do**:
  - Do not implement tab-specific content (delegated to telemetry.rs, logs.rs, controls.rs)
  - Do not use Braille charts

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on tui/mod.rs for types)
  - **Parallel Group**: Sequential within Wave 3
  - **Blocks**: Tasks 10, 11, 12, 13
  - **Blocked By**: Tasks 2, 3, 8

  **References**:
  - Spec §5 (TUI layout, status bar)
  - Spec §6 (disconnect UX)
  - `ratatui` docs: `Layout::default()`, `Constraint`, `Block`, `Paragraph`, `Span`

  **Acceptance Criteria**:
  - [ ] `src/bin/tui/render.rs` exists
  - [ ] Status bar layout function
  - [ ] Active tab dispatch by `ActiveTab` enum
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: Render module compiles
    Tool: Bash
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Finished"
    Evidence: .omo/evidence/task-9-render-check.txt
  ```

  **Commit**: YES (groups with Task 8)

### Wave 4 — TUI Tabs

- [ ] 10. **Create telemetry.rs (Tab 1: raw values + recent history)**

  **What to do**:
  - Create `code/ground-station-frontend/src/bin/tui/telemetry.rs`
  - Implement the Telemetry tab per spec §5 Tab 1:
    - Flight state display (color-coded: Arm=Yellow, Deploy=Red, Touchdown=Green)
    - Altimeter panel: altitude, pressure, temperature
    - GPS panel: lat/lon, altitude, satellite count
    - IMU panel: accel (X/Y/Z), gyro (X/Y/Z), mag (X/Y/Z), temperature
    - Recent History: scrolling text log of telemetry samples with state transition markers
    - Transitions section: arm/deploy/touchdown events with timestamps
  - Read from `AppState`: `latest_record`, `altitude_history`, `gps_history`, `transitions`
  - Color scheme per spec §5 styling: Arm=Yellow, Deploy=Red, Touchdown=Green

  **Must NOT do**:
  - Do not add velocity/acceleration derivation
  - Do not add Braille charts

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11, 12)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3, 4, 6, 7, 8, 9

  **References**:
  - Spec §5 Tab 1 (telemetry layout)
  - `proto/src/record.rs` — Record type fields
  - `proto/src/sensor_data.rs` — AltimeterData, GpsData, ImuData types
  - `ratatui` docs: `Paragraph`, `Gauge`, `Block`, `List`, `Style`, `Color`

  **Acceptance Criteria**:
  - [ ] `src/bin/tui/telemetry.rs` exists
  - [ ] Tab renders: flight state, altimeter, GPS, IMU panels
  - [ ] Recent history shows samples with state transition markers
  - [ ] Color coding: Arm=Yellow, Deploy=Red, Touchdown=Green
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: Telemetry module compiles
    Tool: Bash
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Finished"
    Evidence: .omo/evidence/task-10-telemetry-check.txt
  ```

  **Commit**: YES (groups with Tasks 11, 12)
  - Message: `feat(gs-fe): add TUI tabs — telemetry, logs, controls`

- [ ] 11. **Create logs.rs (Tab 2: log tail placeholder)**

  **What to do**:
  - Create `code/ground-station-frontend/src/bin/tui/logs.rs`
  - Implement the Logs tab per spec §5 Tab 2:
    - Display placeholder text explaining this tab is inactive in M3.2
    - Show the component color-coding reference table (FC-Cyan, SIM-Yellow, GS-BE-White, GS-FE-Green)
    - Wire up rendering code for forward compatibility: when `log_buffer` has content, render it
    - Scrollable view (mouse wheel) — deferred to M3.6 unless trivial
  - For M3.2: backend does not emit `log` messages, so the tab shows a placeholder
  - The WS reader already accepts `log` type messages (no-op handler in state.rs)

  **Must NOT do**:
  - Do not implement M3.6 log aggregation or color-coded rendering
  - Do not add multi-component log forwarding

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 12)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3, 6, 7, 8, 9

  **References**:
  - Spec §5 Tab 2 (logs layout)
  - Spec §5 component color-coding table

  **Acceptance Criteria**:
  - [ ] `src/bin/tui/logs.rs` exists
  - [ ] Shows placeholder text for M3.2
  - [ ] Component color-coding reference table displayed
  - [ ] Forward-compatible: reads from `log_buffer` if content exists
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: Logs module compiles
    Tool: Bash
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Finished"
    Evidence: .omo/evidence/task-11-logs-check.txt
  ```

  **Commit**: YES (groups with Tasks 10, 12)

- [ ] 12. **Create controls.rs (Tab 3: arm, ignite, reconnect + status)**

  **What to do**:
  - Create `code/ground-station-frontend/src/bin/tui/controls.rs`
  - Implement the Controls tab per spec §5 Tab 3:
    - Arm System button `[a]` with last result display
    - Motor Ignition button `[i]` with last result display
    - Connection status panel: FC Status, Latency, Session, Record count
    - Reconnect button `[r]` — restarts WS reader task
    - Keybinds reference section
  - Read from `AppState`: `connected`, `latency_ms`, `session_start`, `record_count`, `last_error`
  - Arm/ignite show `(FC disconnected)` when `connected == false`

  **Must NOT do**:
  - Do not add deploy button (FC-driven only)
  - Do not add simulator lifecycle controls (M3.3)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 11)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3, 6, 7, 8, 9

  **References**:
  - Spec §5 Tab 3 (controls layout)
  - Spec §6 (disconnect UX)

  **Acceptance Criteria**:
  - [ ] `src/bin/tui/controls.rs` exists
  - [ ] Arm/ignite/reconnect buttons with keybindings
  - [ ] Connection status panel
  - [ ] Disconnected state: shows `(FC disconnected)` for arm/ignite
  - [ ] `cargo check -p ground-station-frontend` passes

  **QA Scenarios**:
  ```
  Scenario: Controls module compiles
    Tool: Bash
    Steps:
      1. cargo check -p ground-station-frontend 2>&1
    Expected Result: "Finished"
    Evidence: .omo/evidence/task-12-controls-check.txt
  ```

  **Commit**: YES (groups with Tasks 10, 11)

### Wave 5 — Binary Entry + Polish

- [ ] 13. **Create main.rs (entry point + WS reader spawn + heartbeat poller)**

  **What to do**:
  - Create `code/ground-station-frontend/src/bin/tui/main.rs`
  - Implement per spec:
    - Entry: install panic hook (`utils::logging::install_panic_hook()`)
    - Initialize tracing
    - Create `Arc<AppState>` with `WsBackend` as the backend client
    - **Bootstrap**: Before spawning WS reader, call `backend.fetch_recent_records(Config::HISTORY_MAX_SAMPLES)` to populate RollingHistory with recent records from the session. Push each record to `altitude_history` and `gps_history`. This gives the frontend immediate history on connect.
    - Spawn `run_ws_reader()` task with `CancellationToken`
    - Spawn heartbeat poller task (spec §4.5): ping every PING_INTERVAL
    - Call `run_tui(state)` — the blocking TUI event loop
    - On TUI exit: cancel WS reader task, wait for completion
  - Heartbeat poller per spec §4.5: select! between cancel and ticker.tick()

  **Must NOT do**:
  - Do not add M3.6 log forwarding integration
  - Do not add M3.4 3D rendering initialization

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: NO (final integration task)
  - **Parallel Group**: Sequential
  - **Blocks**: F1-F4
  - **Blocked By**: All Tasks 1-12

  **References**:
  - Spec §4.5 (heartbeat poller)
  - Spec §4.6 (async command dispatch)
  - Spec §4.4 (WS reader spawn)
  - `code/simulator/src/bin/host/main.rs` — existing binary entry pattern
  - `code/ground-station-backend/src/main.rs` — tracing initialization pattern

  **Acceptance Criteria**:
  - [ ] `src/bin/tui/main.rs` exists
  - [ ] Binary compiles and links: `cargo build -p ground-station-frontend` succeeds
  - [ ] Panic hook installed at entry
  - [ ] WS reader spawned with cancellation
  - [ ] Heartbeat poller spawned at PING_INTERVAL
  - [ ] `cargo clippy -p ground-station-frontend` passes (no warnings)

  **QA Scenarios**:
  ```
  Scenario: Full binary build succeeds
    Tool: Bash
    Preconditions: All tasks 1-12 done
    Steps:
      1. cargo check -p ground-station-frontend 2>&1 | tail -5
      2. cargo clippy -p ground-station-frontend 2>&1 | tail -5
      3. cargo build -p ground-station-frontend 2>&1 | tail -5
    Expected Result: All three commands return "Finished" with zero errors/warnings
    Evidence: .omo/evidence/task-13-full-build.txt
  ```

  **Commit**: YES (groups with Task 14)
  - Message: `feat(gs-fe): add binary entry point and README`

- [ ] 14. **Create README.md + update ROADMAP.md**

  **What to do**:
  - Create `code/ground-station-frontend/README.md` following existing patterns:
    - Title: `# ground-station-frontend`
    - Architectural role paragraph (per spec §1)
    - M3.2 scope summary (bullet list of in-scope features)
    - Out-of-scope summary (deferred features with milestone references)
    - Build & run instructions: `cargo build -p ground-station-frontend`, `cargo run -p ground-station-frontend`
    - "See also" section: `spec.md`, `docs/software/spec.md §5.4`, `docs/ROADMAP.md M3.2`
  - Update `docs/ROADMAP.md`:
    - Add GS-FE sub-milestones table at the beginning of M3.2 section
    - Sub-milestones: M3.2a (Foundation), M3.2b (Core Library), M3.2c (TUI), M3.2d (Binary + Polish)
    - Mark all sub-milestones with appropriate status (In progress / Planned)
    - Update the progress table in the Status Summary section
    - Keep inline with the existing ROADMAP structure

  **Must NOT do**:
  - Do not add M3.4 content to README (deferred)
  - Do not change ROADMAP structure for other milestones

  **Recommended Agent Profile**:
  - **Category**: `writing`
  - **Skills**: none needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 13)
  - **Parallel Group**: Wave 5 (can overlap with Task 13)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 2 (Cargo.toml)

  **References**:
  - `code/ground-station-backend/README.md` — existing README pattern
  - `code/simulator/README.md` — another README pattern
  - `docs/ROADMAP.md` — existing roadmap to update

  **Acceptance Criteria**:
  - [ ] `code/ground-station-frontend/README.md` exists
  - [ ] README includes: role, M3.2 scope, build/run, see-also
  - [ ] `docs/ROADMAP.md` has GS-FE sub-milestones in M3.2 section
  - [ ] Sub-milestones: M3.2a-3.2d with status tracking

  **QA Scenarios**:
  ```
  Scenario: README exists and ROADMAP is updated
    Tool: Bash
    Steps:
      1. head -5 code/ground-station-frontend/README.md
      2. grep -A3 "M3.2a" docs/ROADMAP.md
    Expected Result: (1) Shows "# ground-station-frontend"
                        (2) Shows GS-FE sub-milestones
    Evidence: .omo/evidence/task-14-readme-roadmap.txt
  ```

  **Commit**: YES (groups with Task 13)
  - Message: `feat(gs-fe): add binary entry point and README`

---

## Final Verification Wave (MANDATORY)

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in `.omo/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo check -p ground-station-frontend` + `cargo clippy -p ground-station-frontend` + build. Review all changed files for: `unwrap`/`expect` in production paths, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration. Save to `.omo/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | VERDICT`

---

## Commit Strategy

Plan commits:
- **0**: `feat(gs-be): add WS broadcast endpoint at /api/records for GS frontend`
- **1–4**: `feat(gs-fe): add foundation — spec, cargo.toml, config, history`
- **5–7**: `feat(gs-fe): add core library — backend client, state, lib.rs`
- **8–9**: `feat(gs-fe): add TUI core — terminal setup, render dispatch`
- **10–12**: `feat(gs-fe): add TUI tabs — telemetry, logs, controls`
- **13–14**: `feat(gs-fe): add binary entry point and README`

---

## Success Criteria

### Verification Commands
```bash
cargo check -p ground-station-frontend
cargo clippy -p ground-station-frontend
cargo build -p ground-station-frontend
```

### Final Checklist
- [ ] `cargo check -p ground-station-frontend` passes
- [ ] `cargo clippy -p ground-station-frontend` passes (no warnings)
- [ ] All in-scope deliverables exist and compile
- [ ] No out-of-scope features implemented
- [ ] All QA scenario evidence files exist

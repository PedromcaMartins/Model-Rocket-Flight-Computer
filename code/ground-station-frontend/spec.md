# ground-station-frontend — detailed design

- **Status:** draft
- **Implements:** `docs/software/spec.md` §5.4 (GS backend ↔ GS frontend)
- **Depends on:** M3.1 (REST contract), WS amendment (WS endpoint on backend)
- **Target path:** `code/ground-station-frontend/spec.md`

---

## 1. Role

The GS frontend is the operator's UI for the ground station. It connects to the
GS backend over **WebSocket** for live telemetry and log streaming, and over
**REST** for commands (arm, ignite). It never speaks postcard-rpc.

**Architectural constraint** (per `docs/software/spec.md §5.4`): all frontends
never speak postcard-rpc. All data flows through the GS backend.

**In scope (M3.2):**
- WebSocket client receiving telemetry `Record`s and status updates. Protocol schema also defines `log` type (forward compatibility — backend does not emit logs yet).
- REST client for commands (arm, ignite) and ping heartbeat.
- Three-tab TUI: Telemetry (raw values + recent history), Logs (placeholder — deferred to M3.6), Controls.
- Disconnect UX: red banner, dimmed stale data, last-seen timestamp, reconnect button.
- Connection heartbeat with latency display.
- Library/binary split: library owns transport, state, pollers; binary owns UI.

**Out of scope (M3.2):**

| Deferred | Reference | Approach |
|---|---|---|
| 3D rendering (rocket + ground-station model, telemetry overlay) | M3.4 | `ratatui-ratty` inline 3D via Ratty Graphics Protocol |
| Simulator lifecycle controls (Restart/Shutdown — need `sim-gs.sock` first) | M3.3 | REST endpoints on backend |
| Arm/ignite wiring to real FC | M3.3 | Backend routes real → sim-gs.sock |
| Web frontend binary | Future | Reuses same `BackendClient` trait |
| Deploy button — FC-driven only | By design | — |
| Velocity/acceleration derivation from altitude deltas — FC is source of truth | By design | — |
| Traditional Braille time-series charts (alt/vel/acc with min/max/avg) | Replaced by 3D | Superseded by `ratatui-ratty` 3D view |
| Multi-component log forwarding through WS (frontend Logs tab active) | M3.6 | GS-backend aggregates logs from FC, SIM, GS-BE, GS-FE and forwards through WS. Color-coded per component. Under HW deployment: FC/SIM logs absent (on-board only); GS-BE/GS-FE only. |

---

## 2. WS protocol contract

The frontend opens a WebSocket connection to `ws://127.0.0.1:8000/api/records`.

Messages are newline-delimited JSON with a `type` discriminator. The frontend
dispatches on `type` to the appropriate state handler.

| Type | Payload (`data`) | Frequency | Purpose |
|---|---|---|---|
| `record` | Full `proto::record::Record` as JSON | As FC publishes | Live telemetry |
| `log` | `String` — formatted log line: `"<timestamp> [<component>] [<level>] <message>"` | Deferred to M3.6 (placeholder in M3.2) | Log output from all system components: `[FC]`, `[SIM]`, `[GS-BE]`, `[GS-FE]`, etc. GS-backend aggregates component logs where feasible and forwards through WS. Absent under HW deployment (FC logs on-board only). In M3.2, backend does not emit `log` messages — frontend tab shows placeholder. |
| `status` | `{connected, session_start, record_count}` | On connect + on change | Connection health + session meta |

### Wire format (one JSON object per WebSocket message)

```
{"type":"record","data":{"timestamp":1234,"uid":"...","payload":{"Altimeter":{"pressure":101325,"altitude":0.0,"temperature":293.15}}}}
{"type":"log","data":"14:30:01.123 [FC] [INFO] state transition: PreArmed → Armed"}
{"type":"log","data":"14:30:01.456 [SIM] [DEBUG] simulation tick 1234 complete"}
{"type":"log","data":"14:30:01.789 [GS-BE] [WARN] storage flush took 150ms (threshold: 100ms)"}
{"type":"log","data":"14:30:02.000 [GS-FE] [INFO] WS connection established"}
{"type":"status","data":{"connected":true,"session_start":"2026-05-31T14:30:00","record_count":42}}
```

### Connection lifecycle

1. **Connect** — TUI starts → frontend opens WS to `/api/records`.
2. **Streaming** — Backend pushes record/log/status messages as they happen.
3. **Disconnect** — WS close or error → frontend sets `connected=false`, red
   banner + dimmed stale data. The WS reader task returns; no auto-retry.
4. **Reconnect** — User presses `r` (reconnect) → frontend opens a fresh WS.
   The old `AppState` (stale data) stays visible until the first post-reconnect
   messages arrive.

---

## 3. Crate structure

```
code/ground-station-frontend/
├── Cargo.toml
├── spec.md                          ← this document
├── README.md                        ← crate overview
├── src/
│   ├── lib.rs                       ← module decls, re-exports
│   ├── backend.rs                   ← BackendClient trait + WsBackend impl + RestClient
│   ├── state.rs                     ← AppState + WS message dispatch
│   ├── history.rs                   ← RollingHistory<T>: generic ring buffer
│   ├── config.rs                    ← compile-time constants (unit struct)
│   └── bin/
│       └── tui/
│           ├── main.rs              ← entry: init, spawn WS reader, launch TUI
│           ├── mod.rs               ← terminal setup + event loop
│           ├── render.rs            ← layout dispatch by active tab
│           ├── telemetry.rs         ← Tab 1: raw values + recent history
│           ├── logs.rs              ← Tab 2: log tail
│           ├── controls.rs          ← Tab 3: arm, ignite, reconnect + status
│           └── render_3d.rs         ← M3.4: ratatui-ratty 3D viewport (rocket + GS model)
```

---

## 4. Library API

### 4.1 BackendClient trait (`backend.rs`)

```rust
#[async_trait]
pub trait BackendClient: Send + Sync {
    /// Open a WebSocket connection and return a stream of messages.
    async fn connect_ws(&self) -> anyhow::Result<Box<dyn WsMessageStream>>;

    /// POST /api/commands/arm
    async fn arm(&self) -> anyhow::Result<()>;
    /// POST /api/commands/ignite
    async fn ignite(&self) -> anyhow::Result<()>;
    /// POST /api/commands/ping — returns latency in ms
    async fn ping(&self) -> anyhow::Result<f64>;
}

#[async_trait]
pub trait WsMessageStream: Send {
    async fn next(&mut self) -> Option<WsMessage>;
}

pub enum WsMessage {
    Record(proto::record::Record),
    Log(String),
    Status { connected: bool, session_start: String, record_count: u64 },
}
```

One concrete implementation ships with the crate:
- **`WsBackend`** — `tokio-tungstenite` WS + `reqwest` REST.

### 4.2 Concurrency model (`state.rs`)

Three tiers matching each field's size and access pattern:

| Primitive | Fields | Reason |
|---|---|---|
| `AtomicBool` | `connected`, `ping_heartbeat` | Single CPU instruction load/store. No allocation, no refcount. |
| `AtomicU64` | `record_count`, `latency_ms` (fixed-point hundredths) | Trivial type, no heap needed. |
| `Mutex<String>` | `session_start`, `last_error` | Written rarely (once per session or on disconnect). Read on every frame, but `Mutex<String>` is cheap. |
| `ArcSwap<Option<Record>>` | `latest_record` | Lock-free reads on hot render path. Writer clones and atomically swaps. |
| `Mutex<VecDeque<String>>` | `log_buffer` | Append-only log. Writer pushes; reader snapshots. Uncontended. |
| `Mutex<Vec<TransitionEvent>>` | `transitions` | Written once per `FlightState` record. Readers snapshot. Uncontended. |
| `Mutex<RollingHistory<T>>` | `altitude_history`, `gps_history` | One async writer, one sync reader. Simple Mutex is correct. |

```rust
pub struct AppState {
    // -- Connection --
    pub connected: AtomicBool,
    pub ping_heartbeat: AtomicBool,
    pub latency_ms: AtomicU64,
    pub session_start: Mutex<String>,
    pub record_count: AtomicU64,
    pub last_error: Mutex<Option<String>>,

    // -- Telemetry (latest values) --
    pub latest_record: ArcSwap<Option<proto::record::Record>>,

    // -- History (rolling windows) --
    pub altitude_history: Mutex<RollingHistory<f32>>,
    pub gps_history: Mutex<RollingHistory<proto::sensor_data::GpsCoordinates>>,

    // -- Events --
    pub transitions: Mutex<Vec<TransitionEvent>>,

    // -- Logs --
    pub log_buffer: Mutex<VecDeque<String>>,

    // -- Backend client --
    pub backend: Arc<dyn BackendClient>,
}

/// Captured at the instant a FlightState record arrives.
pub struct TransitionEvent {
    pub state: proto::flight_state::FlightState,
    pub time_since_start: f64,
    pub altitude: Option<f32>,
}
```

### 4.3 RollingHistory<T> (`history.rs`)

A rolling-window ring buffer generic over the element type. Parameterized by
window duration (time-based eviction) and max capacity.

```rust
pub struct RollingHistory<T> {
    samples: VecDeque<(Instant, T)>,
    window: Duration,
    max_len: usize,
}

impl<T: Clone> RollingHistory<T> {
    pub fn new(window: Duration, max_len: usize) -> Self;
    pub fn push(&mut self, time: Instant, value: T);
    pub fn latest(&self) -> Option<&T>;
    pub fn snapshot(&self) -> Vec<(Instant, T)>;  // for TUI render
    pub fn len(&self) -> usize;
}
```

Flushes samples older than `window` on each `push()`.

### 4.4 WS reader task

```rust
pub async fn run_ws_reader(
    backend: Arc<dyn BackendClient>,
    state: Arc<AppState>,
    cancel: CancellationToken,
);
```

Opens the WS, reads messages in a loop, dispatches by `type`:

| Message | Handler |
|---|---|
| `record` | Updates `latest_record` (ArcSwap). Pushes to history buffers. Checks for `FlightState` → appends to `transitions`. |
| `log` | No-op in M3.2 (backend does not emit). Handler wired for forward compatibility — when M3.6 activates log forwarding, pushes to `log_buffer` (configurable max lines). |
| `status` | Updates `connected`, `session_start`, `record_count`. |

On WS close/error: sets `connected=false`, captures error, **exits** (no auto-retry).

### 4.5 Heartbeat poller

A dedicated task runs at `PING_INTERVAL`:

```
loop {
    select! {
        _ = cancel.cancelled() => break,
        _ = ticker.tick() => {
            match backend.ping().await {
                Ok(latency) => {
                    state.latency_ms.store((latency * 100.0) as u64, Ordering::Relaxed);
                    state.ping_heartbeat.fetch_xor(true, Ordering::Relaxed);
                }
                Err(_) => state.connected.store(false, Ordering::Relaxed);
            }
        }
    }
}
```

The `ping_heartbeat` toggle drives the status-bar ● blink.

### 4.6 Async command dispatch

Arm/ignite POST requests are dispatched on a background task so they never
block the TUI render loop:

```
User presses 'a' (arm)
  → main loop spawns a tokio task: POST /api/commands/arm, awaits response
  → spawned task writes result to AppState.last_error, AppState.last_cmd_result
  → user sees result on next render frame
```

There is no shared mpsc channel or `BackendCommand` enum. Each command call
spawns its own task which directly invokes the `BackendClient` method and writes
the result to shared state (`last_error`, `last_cmd_result`). This avoids
designing a channel enum and allows each command task to run independently.

---

## 5. TUI layout (M3.2 scope)

### Styling

Arm, Deploy and Touchdown states should be color-coded!
Under telemetry, transitions the text and time is color-coded!
Under recent history, the text corresponding with these states should be color-coded too!
These datapoints according with these states on the charts will be color-coded too!

### Status bar (row 1, always visible)

```
● Connected  |  Ping: 1.2ms  |  Session: 14:30:00  |  Records: 42  |  [1] Telemetry  [2] Logs  [3] Controls  |  [q] quit
```

| Element | Meaning |
|---|---|
| `●` / `○` | Solid green = connected + heartbeats. Blinking = alive. Red = disconnected. |
| `1.2ms` | Latest ping latency (hidden when disconnected) |
| `[1] [2] [3]` | Active tab highlighted. Tab/Shift+Tab or 1-3 to switch. |

### Tab 1 — Telemetry (raw values + recent history text)

```
┌─ Flight State: Armed ──────────────────────────────────────────────┐
│                                                                    │
│  Altimeter:  1234.5 m    Pressure: 85000 Pa    Temp: 22.3 °C       │
│  GPS:        38.7320°, -9.1370°    Alt: 1234.5m    Sats: 12        │
│  IMU:        accel(X:0.1 Y:0.2 Z:9.8)  gyro(X:0.0 Y:0.0 Z:0.0)     │
│              mag(X:1.2 Y:3.4 Z:5.6)  temp: 22.1°C                  │
│                                                                    │
│  ── Recent History (last 30s) ──                                   │
│  T+0.0s    alt:    0.0m    lat: 38.7320   lon: -9.1370             │
│  T+2.1s    alt:   45.0m    lat: 38.7321   lon: -9.1370    ← Arm    │
│  T+5.0s    alt:  320.0m    lat: 38.7325   lon: -9.1369             │
│  T+10.0s   alt:  850.0m    lat: 38.7330   lon: -9.1367             │
│  T+12.7s   alt: 1200.0m    lat: 38.7335   lon: -9.1364    ← Dep    │
│  T+28.3s   alt:    0.0m    lat: 38.7380   lon: -9.1340    ← TD     │
│                                                                    │
│  Transitions:  Arm T+2.1s  │  Deploy T+12.7s  │  Touchdown T+28.3s │
│   Launchpad Altitude: 60.0m   │  Touchdown Altitude: 85.0m         │
└────────────────────────────────────────────────────────────────────┘
```

Recent history: scrolling text log of telemetry samples at acquisition cadence.
Each row shows relative time + key values. State transitions marked inline.
Latest N rows fit available terminal height.

### Tab 2 — Logs

```
┌───────────────────────────────────────────────────────────────────┐
│  14:30:01.123 [INFO] connected to FC on fc-gs.sock                │
│  14:30:02.456 [INFO] subscribed to RecordTopic                    │
│  14:30:03.789 [INFO] record #42 received (Altimeter)              │
│  14:30:04.012 [WARN] storage flush took 150ms                     │
│  ...                                                              │
│ ───────────────────────────────────────────────────────────────── │
│  When active, this tab streams logs from all system components:   │
│    [FC]    Flight Computer — state transitions, sensor events     │
│    [SIM]   Simulator — tick progress, scripted events             │
│    [GS-BE] Ground Station Backend — connection, storage, routing  │
│    [GS-FE] Ground Station Frontend — WS health, command results   │
│                                                                   │
│  Each component is color-coded for quick scanning.                │
│  Under HW deployment: FC logs are on-board only (WS stream        │
│  carries [GS-BE] and [GS-FE] logs only).                          │
│                                                                   │
│  This tab will become active when the log forwarding channel      │
│  (M3.6) is implemented.                                           │
└───────────────────────────────────────────────────────────────────┘
```

The log buffer (`log_buffer: Mutex<VecDeque<String>>`) exists in `AppState` for
forward compatibility. The WS reader accepts `log` type messages (no-op handler
in M3.2 — backend does not emit them yet). When M3.6 activates log forwarding,
the buffer and rendering code are already in place.

Log buffer stores last N lines (default 2000). TUI renders a window.
Scrollable with mouse wheel.

**Component color-coding** (for M3.6 when rendering is active):

| Component | Color | Example |
|---|---|---|
| `[FC]` | Cyan | critical flight events |
| `[SIM]` | Yellow | simulator activity |
| `[GS-BE]` | White | backend operations |
| `[GS-FE]` | Green | frontend diagnostics |

### Tab 3 — Controls

```
┌─ Commands ───────────────────────────────────────────────────────┐
│                                                                  │
│  [a] Arm System                                                  │
│      Last: 503 — simulator not connected    (T+5.2s ago)         │
│                                                                  │
│  [i] Motor Ignition                                              │
│      Last: 503 — simulator not connected    (T+5.2s ago)         │
│                                                                  │
│ ── Connection ────────────────────────────────                   │
│                                                                  │
│  FC Status: ● Connected   Latency: 1.2ms                         │
│  Session:   2026-05-31 14:30:00                                  │
│  Records:   42                                                   │
│                                                                  │
│  [r] Reconnect to backend                                        │
│                                                                  │
│ ── Keybinds ─────────────────────────────────────                │
│  a=arm  i=ignite  r=reconnect  q=quit                            │
└──────────────────────────────────────────────────────────────────┘
```

Each command shows keybinding + label + last result (status + time since).
Reconnect button re-opens the WS connection.

---

## 6. Disconnect UX

When the WS connection closes:

1. **Within one render frame:** Status-bar ● turns red/dim. Latency hidden.
2. **Telemetry tab:** Last-known values dimmed. Stale badge `⚠ STALE — Last seen: Xs ago`.
3. **Controls tab:** Arm/ignite show `(FC disconnected)`. Reconnect button `r` available.
4. **No auto-retry** — operator initiates reconnect.

On reconnect success (user presses `r`):
1. New WS connection opened.
2. First incoming messages populate fresh state.
3. Stale badge disappears, values brighten, ● turns green.

---

## 7. Config (`config.rs`)

```rust
pub struct Config;
impl Config {
    // -- Backend --
    pub const BACKEND_HOST: &str = "127.0.0.1";
    pub const BACKEND_PORT: u16 = 8000;
    pub const WS_PATH: &str = "/api/records";

    // -- TUI --
    pub const TUI_FPS: u16 = 60;
    pub const HISTORY_WINDOW: Duration = Duration::from_secs(30);
    pub const HISTORY_MAX_SAMPLES: usize = 500;
    pub const LOG_BUFFER_CAPACITY: usize = 2000;
    pub const LOG_VISIBLE_LINES: usize = 20;

    // -- Polling --
    pub const PING_INTERVAL: Duration = Duration::from_millis(1000);

    // -- URLs --
    pub fn ws_url() -> String { format!("ws://{host}:{port}{path}", host = Self::BACKEND_HOST, port = Self::BACKEND_PORT, path = Self::WS_PATH) }
    pub fn arm_url() -> String { format!("http://{host}:{port}/api/commands/arm", host = Self::BACKEND_HOST, port = Self::BACKEND_PORT) }
    pub fn ignite_url() -> String { format!("http://{host}:{port}/api/commands/ignite", host = Self::BACKEND_HOST, port = Self::BACKEND_PORT) }
    pub fn ping_url() -> String { format!("http://{host}:{port}/api/commands/ping", host = Self::BACKEND_HOST, port = Self::BACKEND_PORT) }
}
```

---

## 8. Cargo dependencies

```toml
[dependencies]
proto = { version = "*", path = "../proto", default-features = false, features = ["client"] }
utils = { path = "../utils" }

# WebSocket client
tokio-tungstenite = "0.24"
futures-util = "0.3"

# REST client
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }

# TUI
ratatui = "0.29"
crossterm = "0.28"

# Async
tokio = { version = "=1.49", features = ["rt-multi-thread", "time", "sync", "macros", "signal"] }
tokio-util = "0.7"

# Concurrency
arc-swap = "1"

# General
async-trait = "0.1"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
tracing = "0.1"
scopeguard = "1"
```

M3.4 addition:
```toml
# 3D rendering via Ratty Graphics Protocol
ratatui-ratty = "0.2"
```

Workspace member in `code/Cargo.toml`:
```toml
members = [
    ...
    "ground-station-frontend",
]
```

---

## 9. Patterns to follow

- **Config**: unit struct with `pub const` — see `ground-station-backend/src/config.rs`.
- **TUI lifecycle**: crossterm raw mode, alternate screen, `spawn_blocking` for
  render loop, scopeguard panic guard — follow simulator's TUI setup.
- **Error handling**: `anyhow::Result` for fallible operations. No unwrap in
  production paths.
- **Logging**: `tracing` via `utils::logging`.
- **Panic hook**: `utils::logging::install_panic_hook()` at binary entry.
- **Concurrency**: `AtomicBool` for flags, `ArcSwap` for values, `Mutex` for collections.

---

## 10. M3.4 — 3D rendering with ratatui-ratty

- **Approach:** `ratatui-ratty` (Ratty Graphics Protocol) for inline 3D objects.
- **Terminal requirement:** Operator must run the Ratty terminal emulator
  (GPU-rendered, supports inline 3D models via RGP).
- **Fallback:** The Telemetry tab from M3.2 remains available. 3D view is an
  additional tab (Tab 4) or replaces the Telemetry tab when Ratty is detected.

### What it renders

| Object | Source | Behavior |
|---|---|---|
| **Rocket 3D model** | `.obj` or `.glb` file (CAD export) | Position/orientation driven by live telemetry (altitude, GPS, attitude quaternion). Scale by distance. |
| **Launchpad** | Origin axis | Fixed position at launchpad coordinates. |
| **Flight path trail** | Line/curve from GPS history | Growing trail behind the rocket as it flies. |
| **Telemetry overlay** | Rendered as 2D text alongside the 3D viewport | Altitude, velocity, flight state pinned to the 3D scene. |

### Camera modes

| Mode | Behavior | Activation |
|---|---|---|
| **Follow** | Camera tracks the rocket from a configurable offset | Default |
| **Ground-fixed** | Camera stays at launchpad, rocket recedes into the sky | Toggle |
| **Orbit** | User can orbit around the rocket with mouse/keys | M3.5 (mouse support) |

### API sketch

```rust
use ratatui_ratty::{RattyGraphic, RattyGraphicSettings};

// Register the rocket model once at startup
let rocket_model = RattyGraphic::new(
    RattyGraphicSettings::new("models/rocket.obj")
        .id(1)
        .animate(false)
        .scale(0.5)
        .color([0xff, 0x44, 0x00])  // orange body
);
rocket_model.register()?;

// Each frame: update position from telemetry, render into terminal region
rocket_model.update_transform(position, orientation);
(&rocket_model).render(viewport_rect, &mut buf);
```

### Deferred details

The exact camera model, model import pipeline (CAD → `.glb`), and animation
loop are detailed in the M3.4 spec. This section captures the architectural
decision: **use `ratatui-ratty` for 3D, replace traditional Braille charts**
with a live 3D viewport.

---

## 11. Verification

```bash
cargo check -p ground-station-frontend
cargo clippy -p ground-station-frontend

# Manual (full HOST stack, M3.2 scope):
# 1. Start FC-host, simulator, GS backend (with WS endpoint)
# 2. cargo run -p ground-station-frontend
# 3. Confirm: telemetry updates, log streaming, command responses, disconnect UX
```

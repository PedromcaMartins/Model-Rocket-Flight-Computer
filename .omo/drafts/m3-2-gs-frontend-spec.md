# Draft: M3.2 — GS Frontend TUI Spec

## Design Decisions (final)

1. **Live data transport**: WebSocket at `ws://127.0.0.1:8000/api/records`. Records streamed as JSON.
2. **Logs**: Via the same WebSocket connection (multiplexed alongside Records).
3. **Layout**: Tabbed — Telemetry, Logs, Controls tabs.
4. **Tab navigation**: Tab/Shift+Tab AND number keys 1-3.
5. **Manual controls**: Arm + Ignite via new backend REST endpoints. Deploy excluded (FC-driven).
6. **Arm/Ignite endpoints**: Return 503 `{error: "simulator not connected"}` until sim-gs.sock exists.
7. **Backend WS changes**: Separate amendment (not in this frontend spec).
8. **Backend URL**: Config constant `127.0.0.1:8000` (unit struct pattern).

## M3.1 REST Contract (for reference)

| Method | Path | Response |
|---|---|---|
| GET | `/api/status` | `{connected, session_start, record_count}` |
| GET | `/api/records` | `Vec<Record>` |
| GET | `/api/records/latest` | `Record` |
| GET | `/api/logs?<lines>` | `Vec<serde_json::Value>` |
| POST | `/api/commands/ping` | `{latency_ms}` or `{error}` |
| POST | `/api/commands/arm` | (new) — returns 503 for now |
| POST | `/api/commands/ignite` | (new) — returns 503 for now |

## Wire Types (from proto/)

- `RecordData`: Altimeter, Gps, Imu, FlightState, Event, Error
- `FlightState`: PreArmed, Armed, RecoveryActivated, Touchdown

## Architecture

```
WebSocket (ws://127.0.0.1:8000/api/records)
   │
   │  multiplexed stream of JSON messages:
   │    {"type":"record", "data":{...Record...}}
   │    {"type":"log",    "data":"...log line..."}
   │    {"type":"status", "data":{connected, session_start, record_count}}
   │
   ▼
┌─────────────────────────────────────────────┐
│  ground-station-frontend library            │
│                                             │
│  backend.rs: WS client + REST helpers       │
│  state.rs: AppState + message parser        │
│  graph.rs: TimeSeries ring buffers          │
│  config.rs: compile-time constants          │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│  TUI binary (ratatui + crossterm)           │
│                                             │
│  Tab 1: Telemetry (alt/GPS/IMU/flight state)│
│  Tab 2: Logs (WS-streamed log tail)         │
│  Tab 3: Controls (arm, ignite, ping)        │
│  Status bar: connection + session info      │
└─────────────────────────────────────────────┘
```

## Crate structure

```
code/ground-station-frontend/
├── Cargo.toml
├── spec.md                    ← this document
├── README.md                  ← crate overview
├── src/
│   ├── lib.rs                 ← module decls, re-exports
│   ├── backend.rs             ← WebSocket client + REST helpers
│   ├── state.rs               ← AppState + WS message classification
│   ├── graph.rs               ← TimeSeries ring buffers
│   ├── config.rs              ← compile-time constants
│   └── bin/
│       └── tui/
│           ├── main.rs        ← entry: init, spawn WS reader, launch TUI
│           ├── mod.rs         ← terminal setup + event loop
│           ├── render.rs      ← layout dispatch by active tab
│           ├── telemetry.rs   ← Tab 1: telemetry panels
│           ├── logs.rs        ← Tab 2: log tail panel
│           └── controls.rs    ← Tab 3: arm/ignite/ping
```

## WS message format (contract with backend)

All messages are newline-delimited JSON. Each line has a `type` discriminator:

```
{"type":"record","data":{...serialized proto::record::Record...}}
{"type":"log","data":"2026-05-31T14:30:01 [INFO] connected to FC"}
{"type":"status","data":{"connected":true,"session_start":"2026-05-31T14:30:00","record_count":42}}
```

The frontend reads from the WS stream, matches on `type`, and dispatches to the appropriate state handler.

## Data flow

1. **WS reader task** (tokio::spawn): opens WS to `/api/records`, reads lines,
   parses JSON, dispatches by `type` to `state::on_record()`, `state::on_log()`,
   `state::on_status()`.
2. **REST client**: on-demand for arm, ignite, ping (POST) + status fetch on
   initial connect.
3. **TUI render loop** (crossterm event loop, ~10fps): reads `AppState`, renders
   active tab.
4. **Disconnect**: WS close → `connected=false`. Telemetry tab shows:
   - Red banner "FC DISCONNECTED"
   - Last-known values dimmed with "Last seen: Xs ago"
   - Controls tab shows Reconnect button (no auto-retry)

## TUI layout (per tab)

**Status bar** (always visible, 1 row at top):
```
● Connected  |  Ping: 1.2ms  |  Session: 14:30:00  |  Records: 42  |  1: Telemetry  2: Logs  3: Controls  |  q: quit
```

**Tab 1 — Telemetry**:
```
┌──────────────────────────────────────────────────────────────┐
│ Flight State: Armed                                          │
│                                                              │
│ Altimeter:  1234.5 m   |  Pressure: 85000 Pa  |  22.3 °C     │
│ Accel:      9.81 m/s²                                        │
│                                                              │
│ GPS:  38.7320°, -9.1370°  |  Satellites: 12                  │
│                                                              │
│ IMU:  accel(X:0.1 Y:0.2 Z:9.8)  gyro(X:0.0 Y:0.0 Z:0.0)      │
│       mag(X:1.2 Y:3.4 Z:5.6)  temp: 22.1°C                   │
│                                                              │
│ Transitions:  ◆ armed T+2.1s  │  ★ deployed T+12.7s         │
│   Launchpad Altitude: 60.0m   │  Touchdown Altitude: 850.0m  │
└──────────────────────────────────────────────────────────────┘
```

**Tab 2 — Logs**:
```
┌──────────────────────────────────────────────────────────────┐
│ 14:30:01.123 [INFO] connected to FC                          │
│ 14:30:02.456 [INFO] subscribed to RecordTopic                │
│ 14:30:03.789 [INFO] record #42 received                      │
│ 14:30:04.012 [WARN] storage write took >100ms                │
│ ...                                                          │
└──────────────────────────────────────────────────────────────┘
```

**Tab 3 — Controls**:
```
┌──────────────────────────────────────────────────────────────┐
│ FC Commands:                                                 │
│   [a] Arm System             Last: 200 OK  (T+2.1s)          │
│   [i] Motor Ignition         Last: 503 (simulator offline)   │
│                                                              │
│ Connection:                                                  │
│   [r] Restart Simulator                                      │
│   [s] Shutdown Simulator                                     │
│   [x] Reconnect to backend                                   │
└──────────────────────────────────────────────────────────────┘
```

## Key design choices

1. **proto as dependency** for `Record` JSON deserialization (no transport features).
2. **tokio-tungstenite** for WebSocket client.
3. **reqwest** only for REST commands (arm/ignite/ping) — not for live telemetry or logs.
4. **ArcSwap** for last-known-value fields (lock-free TUI reads every frame).
5. **Mutex<TimeSeries>** for graph history (uncontended — one writer, one reader).
6. **Tab/Shift+Tab + 1-3** for navigation.
7. **WS message format** uses `type` discriminator for Record/Log/Status multiplexing.

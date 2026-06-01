# ground-station-backend

REST/JSON server + telemetry storage for the ground station.

**Architectural role** (per `docs/software/spec.md`):
- Postcard-rpc **client** on `fc-gs.sock` — subscribes to `RecordTopic` for FC telemetry.
- NDJSON session storage with in-memory record cache (`logs/gs_records/<timestamp>/records.ndjson`). REST reads from cache; NDJSON is the durable journal.
- REST/JSON API consumed exclusively by the GS frontend (never speaks postcard-rpc directly).

**M3.1 scope:**
- FC-facing only — no `sim-gs.sock` connection (deferred to M3.3).
- Automatic reconnection — the FC client loops indefinitely, reconnecting after [`Config::RECONNECT_INTERVAL`](src/config.rs) on any failure or disconnect.

## REST API

| Method | Path | Description |
|---|---|---|
| GET | `/api/status` | FC connection state + session record count |
| GET | `/api/records` | All records from current session |
| GET | `/api/records/latest` | Most recent record |
| GET | `/api/logs` | Recent GS-side log lines |
| POST | `/api/commands/ping` | Send ping to FC (future) |

Config constants and logging details live in the `src/main.rs` rustdoc.

## Build & run

```bash
# From the workspace root (code/)
cargo build -p ground-station-backend

# Run (FC must be listening on fc-gs.sock)
cargo run -p ground-station-backend
```

Requires `proto` features `client` + `transport-ipc`.

## See also

- `docs/software/spec.md §5.2` — FC ↔ GS boundary contract
- `docs/ROADMAP.md M3.1` — implementation milestone
- `docs/software/spec.md §9.2` — proto consumer feature map

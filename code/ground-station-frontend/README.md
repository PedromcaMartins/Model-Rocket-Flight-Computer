# ground-station-frontend

ratatui TUI operator interface for the ground station.

**Architectural role** (per `docs/software/spec.md §5.4`):
- **WebSocket client** on `ws://127.0.0.1:8000/api/records` — receives live telemetry `Record`s and status updates.
- **REST client** — issues arm, ignite, and ping commands to the GS backend.
- **Never speaks postcard-rpc** — all data flows through the GS backend.

**M3.2 scope:**
- Three-tab TUI: Telemetry (raw values + recent history), Logs (placeholder), Controls (arm/ignite/reconnect).
- Disconnect UX: red banner, dimmed stale data, reconnect button.
- Connection heartbeat with latency display (blinking ● indicator).
- Library/binary split — library owns transport, state, pollers; binary owns UI.

**Out of scope (M3.2):**
- 3D rendering (ratatui-ratty) — deferred to M3.4.
- Log forwarding through WS — deferred to M3.6.
- Simulator lifecycle controls — deferred to M3.3.

## Build & run

```bash
# From the workspace root (code/)
cargo build -p ground-station-frontend

# Run (GS backend must be running)
cargo run -p ground-station-frontend
```

## Controls

| Key | Action |
|---|---|
| `q` / `Ctrl+C` | Quit |
| `Tab` / `Shift+Tab` | Switch tabs |
| `1` / `2` / `3` | Select tab (Telemetry / Logs / Controls) |
| `a` | Arm system |
| `i` | Motor ignition |

WS reconnect is automatic — no manual keybind needed.

## See also

- `spec.md` — detailed design for this crate
- `docs/software/spec.md §5.4` — GS backend ↔ GS frontend boundary contract
- `docs/ROADMAP.md M3.2` — implementation milestone

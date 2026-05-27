# `proto` — shared wire vocabulary

`no_std` crate holding the single postcard-rpc Topic / Endpoint / message-type
contract shared by every component in the stack. All telemetry, commands,
simulator peripheral data, and lifecycle messages cross process boundaries
using the definitions in this crate.

**Architectural constraints:** `docs/software/spec.md §9`

## Consumer quick reference

| Binary | Features | Transport |
|---|---|---|
| `flight-computer-host` | `host` | `transport-ipc` (server on both sockets) |
| `simulator` (host binary) | `host` | `transport-ipc` (client) |
| `ground-station-backend` | `client` + `transport-ipc` | `transport-ipc` (client) |
| PIL firmware (`cross-*`) | `pil` | USB serial (via `nusb`) |
| HW firmware (`cross-*`) | `hw` | None (embedded) |

## Crate layout

```
src/
├── lib.rs              ← Feature gates, topics!, endpoints! macros, re-exports
├── client.rs           ← PostcardClient + PostcardError (gated on `client`)
├── transport/
│   └── ipc.rs          ← InterprocessWireTx / InterprocessWireRx
├── sensor_data.rs      ← AltimeterData, GpsData, ImuData
├── actuator_data.rs    ← ActuatorStatus, LedStatus
├── flight_state.rs     ← FlightState enum
├── event.rs            ← Event types
├── error.rs            ← Wire-level error types
├── newtypes.rs         ← Unit-wrapper newtypes (uom-backed)
├── record/
│   ├── mod.rs          ← Record enum
│   └── tick_hz.rs      ← GlobalTickHz
└── wire.rs             ← Server accept / client connect helpers
```

## See also

- `src/lib.rs` — feature flags, full endpoint/topic definitions, developer guide
- `docs/software/spec.md §9` — architectural constraints on feature gating
- `docs/software/spec.md §9.2` — consumer feature map (architecture view)
- `Cargo.toml` — exact dependency and feature flag definitions

# `Software/` — Subsystem-level architecture

This folder owns architecture and interface design for individual subsystems and for the contracts that cross subsystem boundaries. Implementation details (which crate, which transport, which channel type) belong next to the artifact in `code/<crate>/` — see [`../README.md`](../README.md#scope-of-this-folder) for the split.

Each doc here answers *what* the subsystem does and *why*, plus *what crosses its public boundaries*. It does not pin down *how* the inside is built.

## Current contents

| Path | Scope |
|---|---|
| [`flight-computer.md`](flight-computer.md) | FC library design goals and invariants: peripheral-agnostic traits, runtime-agnostic async, architecture-agnostic (RISC-V / ARM / x86), single postcard-rpc vocabulary across all modes. Read this first. |
| [`deployment-modes.md`](deployment-modes.md) | The three deployment topologies (HW, HOST, PIL) — what each is for, which binaries/libs participate, what crosses the wire. |
| [`fc-simulator-interface.md`](fc-simulator-interface.md) | The peripheral-trait contract between the FC and the simulator (sensors, arming, deployment, LEDs) and its postcard-rpc implementation across HOST and PIL. |

## See also

- [`../REQUIREMENTS.md`](../REQUIREMENTS.md) — `[SW-5*]` covers the test-mode contract these documents implement.
- [`../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md`](../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md) — the decision to split host into per-role binaries using postcard-rpc over interprocess local sockets.

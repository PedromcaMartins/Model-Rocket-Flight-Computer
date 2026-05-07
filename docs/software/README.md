# `Software/` — Subsystem-level architecture

The software architecture lives in [`spec.md`](spec.md) — one file, top-to-bottom: goals, components, boundaries, cross-component invariants, all subsystems (FC library, simulator, host IPC, observability, crash policy), known limitations, open questions, and a HOST scenario walk-through. Implementation details (which crate, which transport, which channel type) belong next to the artifact in `code/<crate>/`; see [`../README.md`](../README.md#scope-of-this-folder) for the split.

## See also

- [`../REQUIREMENTS.md`](../REQUIREMENTS.md) — `[SW-5*]` covers the test-mode contract this spec implements.
- [`../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md`](../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md) — *why* postcard-rpc + binary split was chosen. The contract it produced lives in [`spec.md` §8](spec.md#8-host-ipc).
- [`../ROADMAP.md`](../ROADMAP.md) — implementation milestones for the binary split.

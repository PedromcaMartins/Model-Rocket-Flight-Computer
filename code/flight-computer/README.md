# flight-computer

The FC library — the software core of the rocket. Library crate only; no binary. Binaries that link it supply the peripheral implementations and the async executor.

## Design invariants

- **Peripheral-agnostic.** The library never imports a driver, simulator type, or transport crate. All hardware interaction goes through traits in `src/interfaces/`; the implementation is supplied at link time by the binary.
- **Runtime-agnostic async.** Uses `async fn` throughout with no dependency on a specific executor. Embassy runs it on HW and PIL; Tokio runs it on host.
- **Architecture-agnostic.** Compiles for RISC-V (ESP32), ARM Cortex-M (STM32), and x86/x64 (host). Must stay `no_std`-clean for embedded targets.
- **Single wire vocabulary.** All telemetry and commands use the types and postcard-rpc endpoint/topic definitions in `proto/`. The same definitions work across HW, PIL, and HOST — only the transport adapter differs.
- **Event-driven FSM.** The flight state machine has no loop rate; transitions execute on incoming events only.
- **No shutdown path on production targets.** Sim-control / orchestration shutdown logic is gated behind the `impl_host` / `impl_software` features or binaries, and must not appear in HW firmware artefacts. Production recovery is reset or watchdog only.
- **Not a framework.** This is the flight software for this rocket, not a reusable domain library. Generalising it is out of scope.

## Features

| Feature | What it enables |
|---|---|
| `impl_embedded` | Real hardware drivers (`embedded-hal`) — used in HW binaries |
| `impl_software` | Simulator-fed peripherals via postcard-rpc over USB — used in PIL firmware |
| `impl_host` | Simulator-fed peripherals via postcard-rpc over interprocess socket — used in the host binary |
| `std` | Standard library (required by `impl_software` and `impl_host`) |
| `log` | Logging via the `log` crate (default for host/test builds) |
| `defmt` | Logging via `defmt` (for embedded targets) |

Default features include all three `impl_*` flags — suitable for host development. Embedded binaries disable defaults and enable only `impl_embedded` + `defmt`.

## Platform dependencies

The linking binary must provide:
- An `embassy-time` driver (time source for `Ticker` and timeouts).
- A `critical-section` implementation (required by `embassy-sync` inter-task primitives).

## Architecture docs

- [`docs/software/spec.md`](../../docs/software/spec.md) — the consolidated software spec. FC library design goals and trait system in §6, FC ↔ Sim peripheral contract in §5.1, IPC topology (FC is server on `fc-sim.sock` and `fc-gs.sock`) in §8, deployment modes (HW / HOST / PIL) in §2, observability and crash policy in §9–§10.

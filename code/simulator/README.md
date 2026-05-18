# simulator

Standalone physics simulator for the flight computer. Runs the rocket flight
model and drives the FC over postcard-rpc by publishing sensor data and reacting
to FC actuator commands — closing the sensor → FC FSM → deployment loop with no
ground station present.

The simulator is the postcard-rpc **client**.

| Binary | Transport | Use |
|---|---|---|
| `host` | interprocess socket `fc-sim.sock` | HOST mode (FC + sim as separate host processes) |
| `pil`  | USB | PIL mode (FC on prod MCU, sim on host) |

Both binaries share one library; they differ only in how the client transport
is constructed.

MVP scope: 1D parabolic physics, compile-time scripted scenario, read-only
ratatui TUI, structured JSON logging. No GS interaction, no config file, no
interactive controls (deferred — see `docs/ROADMAP.md`).

Detailed design: [`spec.md`](spec.md).

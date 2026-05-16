# flight-computer-host

The HOST deployment mode binary. Binds two local sockets, accepts a connection
on each (simulator first, then GS backend), then hands both postcard-rpc servers
to the FC library.

| Socket | Connects to |
|---|---|
| `fc-sim.sock` | Simulator |
| `fc-gs.sock` | GS backend |

Socket names use `GenericNamespaced` (abstract namespace on Linux, `\\.\pipe\*`
on Windows) for OS-conformant naming with no leftover files.

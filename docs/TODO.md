- [ ] Change type of Ping Message to `PingRequest` and `PingResponse` in telemetry and the rest of the libs!
- [ ] Use `xtask` crate to manage build and run tasks

# Flight Computer lib

- [X] Rename quantities used in telemetry messages e.g. `pub type Altitude = Length`
- [X] Melhorar `FileSystemEvent`
- [X] Deployment System
- [ ] ApogeeDetector
    - [X] Add Configuration to models
    - [ ] Testing in model
- [ ] Landing detector in FSM
    - [X] Add Configuration to models
    - [ ] Testing in model
- [X] Tracing with sync and async
    - [ ] Testing for tracing structs
- [ ] Add state to flight computer! 
- [ ] Document and reduce use of Panics in functions
- [ ] postcard task that receives a server and executes in loop the postcard server run method!

# Host Flight Computer lib

- [X] Create host drivers lib to be used in Flight Computer lib tests / Create test drivers in flight computer lib exclusive to testing
- [X] Create Simulator
    - [X] Altitude
    - [X] Gps
    - [X] Imu
    - [X] Logging with Tokio Console + Tracing
    - [~] Add Custom Timer Driver with std + advance()
        - Not necessary... 
    - [ ] Integration with ground station lib
    - [ ] Physics Engine + config (1D)
    - [ ] Manual triggers
        - [ ] Fire Ignitor
        - [ ] Deploy Recovery (Parachute)
        - [ ] Trigger Arm
    - [ ] Automatic triggers (Scripted Events)
    - [ ] Fault Engine + config
- [ ] Add [BlockDevice](https://docs.rs/embedded-sdmmc/0.9.0/embedded_sdmmc/trait.BlockDevice.html) implementation for filesystem - then mount the filesystem as FATFS - what is tested using this?... 
- [ ] Add [Sguaba](https://docs.rs/sguaba/latest/sguaba/index.html) for reference frames

# Ground Station

- [ ] add backend binary for serial port connection (not just usb) with the embedded target!
- [ ] Develop a TUI front-end 
    - [ ] Display PostCard Messages
    - [ ] Display log messages
        - [ ] Open logs dir / current log
    - [ ] Display simulator
        - [ ] start/stop/restart simulation 
        - [ ] tweak simulation options
        - [ ] send manual events
- [ ] Implement Trace Parser that can generate somewhat flame-graphs
- [ ] Store all data locally on disk/ memory
- [ ] Use REST API with JSON serialization for communication between backend and frontend

# New Tasks

- [ ] Improve error handling -> ThisError
- [ ] Implement Events, Errors, and Stats! -> Add Stats struct, stats task, event/error channel, ...
    - [ ] Standerdize `type Error: core::fmt::Debug` in traits!
    - [ ] Errors + Events should have a severity level
    - [ ] Move GPS Error to proto + adapt it
- [ ] Create Config that can be changed at runtime via Ground Station: Add struct RuntimeConfig + postcard endpoints + atomic watch + change config to add this
    - [ ] Improve Config struct naming
- [ ] Crate `broadcast_record(Record)` global function -> send to Storage + send to Ground Station
    - [ ] Storage should store all data, compression allowed (for events, errors)
    - [ ] FC should send to Ground Station sparse data (e.g. only critical errors, events, latest sensor data + stats every n milliseconds) to minimize bandwidth use
- [ ] Check core/state_machine
- [X] Decouple Postcard from core!: Should only require Value to send + Topic!
    - ~~[ ] Postcard Sender needs to be static + set in runtime globally!~~
    - ~~[ ] global function `send_to_ground_station<T: Topic>(value: &T::Message)`~~
    - [X] Groundstation thread receives records!
    - [X] Contains atomic seq number
- [ ] Implement Read from Storage -> Groundstation command activates it! ~~Implement Iterator over Record~~
- [X] Make `sim_filesystem_led` generic to any Led!
- [X] Use select in sensor reading to wait at the same time for sensor data and for next iter signal! -> that way every tick, new data is read if available!
- [ ] Use TraceSync + Async for benchmarks in functions!
- [X] Make sensor tasks generic over sensor devices!
- ~~[ ] write! -> uwrite!~~
- [X] Add tests for core modules!
- [ ] If test, dont use `must_use`!
- [ ] TraceAsync - linter config
- [ ] Trace - parser to get info back! uder feature wall
    ```Rust
    pub enum Stage {
        Sync(u64),
        Await(u64),
    }
    ```
- [X] Rename `SensorDevice` trait to Sensor

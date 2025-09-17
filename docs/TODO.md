# Flight Computer lib

- [X] Rename quantities used in telemetry messages e.g. `pub type Altitude = Length`
- [X] Melhorar `FileSystemEvent`
- [X] Deployment System
- [X] ApogeeDetector
    - [X] Add Configuration to models
    - [ ] Testing in model
- [X] Landing detector in FSM
    - [X] Add Configuration to models
    - [ ] Testing in model
- [ ] Document and reduce use of Panics in functions

# Host Flight Computer lib

- [X] Create host drivers lib to be used in Flight Computer lib tests / Create test drivers in flight computer lib exclusive to testing
- [X] Create Simulator
    - [X] Altitude
    - [X] Gps
    - [X] Imu
    - [X] Logging with Tokio Console + Tracing
    - [ ] Add Custom Timer Driver with std + advance()
    - [ ] Physics Engine + config (1D)
    - [ ] Manual triggers
        - [ ] Fire Ignitor
        - [ ] Deploy Parachute
        - [ ] Trigger Arm
    - [ ] Automatic triggers (Scripted Events)
    - [ ] Fault Engine + config
    - [ ] Integration with ground station lib

# Ground Station

- [ ] Develop a TUI front-end 
    - [ ] Display PostCard Messages
    - [ ] Display log messages
        - [ ] Open logs dir / current log
    - [ ] Display simulator
        - [ ] start/stop/restart simulation 
        - [ ] tweak simulation options
        - [ ] send manual events

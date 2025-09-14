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

- [ ] Create host drivers lib to be used in Flight Computer lib tests / Create test drivers in flight computer lib exclusive to testing
- [ ] Create Simulator
    - [ ] Logging with Tokio Console + Tracing
    - [ ] Physics Engine + config
    - [ ] Altitude
    - [ ] Gps
    - [ ] Imu
    - [ ] Manual triggers
        - [ ] Fire Ignitor
        - [ ] Deploy Parachute
        - [ ] Trigger Arm
    - [ ] Automatic triggers (Scripted Events)
    - [ ] Fault Engine + config

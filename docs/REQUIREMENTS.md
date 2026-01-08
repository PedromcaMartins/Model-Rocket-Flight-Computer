# Requirements

## Development Methodology

- [DEV-1] Open Rocket / RocketPy simulations must be performed to evaluate and define further Rocket System requirements.
- [DEV-2] The rocket must be fully designed in CAD software.
- [DEV-3] Recovery systems must be tested: system activation testing and load testing.
- [DEV-4] Avionics systems must be tested.
- [DEV-5] Propulsion or Structural systems don't have to be tested before launch.

## Rocket system

- [ROCKET-1] The rocket must use at max, an E-class motor and must be commercially available.
- [ROCKET-2] The rocket must have passive stabilization.
- [ROCKET-3] The rocket must have a recovery system.
- [ROCKET-4] The rocket must have an avionics system to log flight data.
- [ROCKET-5] Parts should be made reusable.
- [ROCKET-6] Parts should be strong, but affordable.
- [ROCKET-7] Parts should be 3D printed, if possible.
- [ROCKET-8] Fuselage should be made out of cardboard tube: light, affordable, easily modified.

## Software subsystem

- [SW-1] The system must contain sensors on-board.
    - [SW-1A] The sensors must include: imu, altimeter/barometer, GPS.
- [SW-2] The system must activate the recovery system at apogee.
    - [SW-2A] The system needs to detect apogee.
- [SW-3] The system must be armed by a human.
    - [SW-3A] Arming the rocket enables the flight state detector, and recovery systems.
    - [SW-3B] The system must provide feedback to the user about the arming status.
- [SW-4] The system must display and store persistently all sensor data, events, and errors.
    - [SW-4A] The storage must be non-volatile memory (e.g., Storage, EEPROM, Flash memory).
    - [SW-4B] The system must provide a way to download the stored data after flight.
    - [SW-4C] The system must provide a real-time display of sensor data while in flight.
- [SW-5] The system must provide a test suite: hardware tests, unit tests, flight sequence tests, full system tests: testing successful and unsuccessful scenarios.
    - [SW-5A] The test suite must provide SITL (Software In The Loop) simulations, for testing flight sequences and FC behavior without hardware.
        - [SW-5A1] The SITL simulations must provide a way to simulate sensor data inputs.
        - [SW-5A2] The SITL simulations must provide a way to simulate recovery system activation.
        - [SW-5A3] The SITL simulations must provide a way to simulate different flight phases.
        - [SW-5A4] The SITL simulations must provide a way to simulate errors and failures.
        - [SW-5A5] The SITL simulations must provide a way to log simulated flight data.
        - [SW-5A6] The SITL simulations must provide a way to visualize flight data (e.g., graphs, charts).
    - [SW-5B] The test suite must provide a way to mock hardware components.
    - [SW-5C] The test suite must provide a way to run tests automatically.
    - [SW-5D] The test suite must provide code coverage reports.
    - [SW-5E] The test suite must provide performance benchmarks.
- [SW-6] The code should be modular and allow for future expansion, as new missions and requirements are added to the project.
    - [SW-6A] The Public API for crates used should be documented.
    - [SW-6B] The flight computer code should be hardware-agnostic, allowing for easy migration to different hardware platforms in the future.
- [SW-7] The system should be developed using Rust programming language.
- [SW-8] The system should display status information using LEDs.
    - [SW-8A] The system should use different LED colors or patterns to indicate different statuses (e.g., armed, flight phase, errors).
    - [SW-8B] The system should use an LED per component to indicate its status and events (e.g., sensor data valid, recovery system armed, flight state detected, data logging active).
- [SW-9] The system must implement a ground-station communication protocol.
    - [SW-9A] The system must transmit real-time telemetry data to the ground station during flight.
    - [SW-9B] The system must receive commands from the ground station (e.g., arm/disarm, request data).
    - [SW-9C] The system must log all transmitted and received data for post-flight analysis.
    - [SW-9D] This system must also be used for SITL and HITL testing, for sendind, receiving, logging, and visualising telemetry and simulated data.

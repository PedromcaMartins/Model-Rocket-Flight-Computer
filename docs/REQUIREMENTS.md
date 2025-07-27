# Requirements

## Development

- Open Rocket / RocketPy simulations must be performed to evaluate and define further Rocket System requirements. 
- The rocket must be fully designed in CAD software. 

- Recovery systems must be tested: system activation testing and load testing. 
- Avionics systems must be tested. 
- Propulsion or Structural systems don't have to be tested before launch. 

## Rocket system

- The rocket must use at max, an E-class motor. 
- The rocket must have passive stabilization. 
- The rocket must have a recovery system. 
- The rocket must have an avionics system to log flight data. 

- Parts should be made reusable. 
- Parts should be strong, but affordable. 
- Parts should be 3d printed, if possible. 

- Fuselage should be made out of cardboard tube: light, affordable, easily modified. 

## Software system

- The system must have an IMU, GPS and Barometer on-board. 
- The system must activate the recovery system at apogee. 
    - The system needs to detech apogee. 
- The system must be armed by a human. 
    - Arming the rocket enables the flight state detector, and recovery systems. 

- The system must display and store persistently all (possible) sensor data, and any errors. 
    - The sensor data stored persistently must be benchmarked. 
- The system must provide a test suite: hardware tests, unit tests, flight sequence tests, full system tests: testing successful and unsuccessful scenarios. 

- The code should be modular and allow for future expansion, as new missions and requirements are added to the project. 

# Motivation 

This repository is the result of an ongoing project to develop a Model Rocket, while learning from many fields of engineering I find interesting. 

My background is in Computer Science, and I've got 2 years of experience designing software for a rocket team @RED, Portugal. 
I want to challenge myself by using Rust, a language I've been learning since 2024, for the flight computer, and some of the Ground-station stack, if not all... 

I also want to take this opportunity to learn more about designing model rocket design / simulator, get experience with CAD modeling by modeling and 3d printing parts for the model rocket, electronics and PCB design, and eventually control algorithms :)

# Repository layout

```
# IDE Configuration settings
.vscode/
.zed/

# Rust Crates: flight computer hardware-agnostic no_std library, telemetry messages, ground-station, and target-host testing (HITL)
crates/*

# Rust Crates for Embedded Targets (Firmware + Board Testing)
# Contains bring-up code for initial testing and diagnosis, IO configuration, and flight computer binary code
crates/cross-esp32-s3/*
crates/cross-nucleo-f413zh/*

# Documentation (Architecture, Requirements, ...)
docs/
./README.md

# Miscelaneous
datasheet/
gps_configuration/

```



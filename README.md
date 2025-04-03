# cubesat-rs

<img src="logo/logo.png " width="300" style="display: block; margin: auto;">

The goal of this personal project is developing a demonstrable embedded system simulating a key satellite subsystem (power management), running on an nRF52840 DK, communicating with a Rust-based command-line ground station application via CCSDS-compliant radio messages.
The focus is on the software side of things, not simulation accuracy.

There are three separate sub-projects:
- The embedded software for the nRF52840 DK
- The firmware for the nRF52840 USB Dongle
- The ground station software


### Plan
##### CubeSat EPS "simulator" with CCSDS-Compliant Telemetry
- [ ] Setup project and embedded Rust dev environment
- [ ] Investigate and prototype a simplified CubeSat EPS - Electrical Power System
- [ ] Test and pick `ccsds` or `spacepackets` for data transmission
- [ ] Modify or rewrite nRF52840 Dongle's firmware
- [ ] Test basic data transmission and reception

##### Ground station software 
- [ ] Study and implement TLE parsing and propagation (`sgp4`)
- [ ] Implement basic attitude representation and control
- [ ] Visualize the satellite's orbit with `kiss3d`
- [ ] Visualize telemetry with `plotters`
- [ ] Integrate Dongle
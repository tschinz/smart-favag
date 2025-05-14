# smart-favag

Reviving a vintage **FAVAG Nebenuhr** (slave clock) with modern technology.

## Project Goal
The **smart-favag** project breathes new life into a historical FAVAG slave clock by using a modern microcontroller and embedded software. The aim is to maintain the charm of the old electromechanical clock while adding contemporary features like automatic time synchronization and intelligent control.

## Hardware Overview

| Component                     | Description                                                                 |
| ----------------------------- | --------------------------------------------------------------------------- |
| **Microcontroller**           | Raspberry Pi Pico W (RP2040) or Pico 2W (RP2350)                            |
| **Motor Driver Shield**       | DS293 Motor Driver to drive the clock's mechanical stepper mechanism        |
| **Power Supply (UPS)**        | 18650 Li-Ion battery with UPS functionality for autonomous operation        |
| **3D Printed Backplate**      | Custom 3D-printed mounting plate for compact integration of components      |

## Features

- [x] Standalone operation with UPS battery support
- [x] Low power operation using sleep modes
- [ ] Normal time display (impulse control every 60 seconds)
- [ ] Fast forward mode (e.g., initial setup or catch-up after power loss)
- [ ] Automatic time synchronization via WiFi (NTP or HTTP API)

## Software Stack

- **Rust 🦀**: Modern, safe, and performant systems programming language.
- **[embassy-rs](https://embassy.dev/)**: Async embedded framework for Rust, used for structured concurrency, timing, and hardware abstraction.

## Roadmap

- [x] Basic impulse driving of FAVAG mechanism
- [ ] WiFi setup and time acquisition
- [ ] Configurable synchronization intervals
- [ ] Web interface for manual control & diagnostics
- [ ] EInk display for time and status


## License

MIT License

## Credits & Inspiration

- The timeless design of the **FAVAG Nebenuhr**.
- The open-source community around **Rust** and **embassy-rs**.
- Special thanks to all contributors keeping vintage tech alive with modern tools.

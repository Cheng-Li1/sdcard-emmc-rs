# Async, OS-Agnostic SD/MMC Card Driver in Rust

This repository contains an SD/MMC card driver written in pure Rust. Leveraging Rust's `async/await` features, it provides non-blocking interfaces for read, write, and erase operations.

The driver is designed to be OS-agnostic. All platform-specific dependencies (e.g., timers, voltage control, power cycling) are provided to the driver by passing trait objects during initialization. This design allows for seamless integration into any operating system or bare-metal environment.

While initially developed for [LionsOS](https://github.com/au-ts/lionsos), it can be readily adapted for other platforms.

**Note:** This driver is in the early stages of development and does not yet implement the full feature set found in mature stacks like the Linux MMC subsystem.

---

## Features

### Supported
- **Card Types:** SDHC / SDXC
- **Bus Speeds:** High Speed (SDHS) / UHS-I
- **Core Operations:** Asynchronous Read, Write, and Erase

### Not Yet Implemented or Tested
- **Card Types:** SDSC / SDUC, eMMC
- **Interface Modes:** SPI mode
- **Bus Speeds:** Speed classes higher than UHS-I (e.g., UHS-II, UHS-III)
- A comprehensive set of features found in mature MMC stacks.

---

## Platform Support

| Platform       | Status          |
| :------------- | :-------------- |
| Odroid C4      | ✅ Supported    |
| sdhci-zynqmp   | 🚧 In Progress  |

---

## Adding Support for a New Platform

Porting the driver to a new platform involves implementing the hardware-specific logic. Follow these steps:

1.  **Familiarize Yourself:** Study the driver's architecture and review the reference implementation for the Odroid C4 to understand the core components.
2.  **Integrate:** Add the driver as a dependency in your target OS or bare-metal environment.
3.  **Implement the HAL:** Create a new hardware abstraction layer (HAL) for your platform by implementing the `SdmmcHardware` trait. This trait defines the interface for all hardware-specific operations.
4.  **Build with Logging:** Compile your HAL and the core protocol crate with the `dev-logs` feature enabled to get detailed diagnostic output.
5.  **Test and Verify:** Run the driver on your hardware. Analyze the logs to verify that the initialization sequence and command responses are correct.

---

## Design Philosophy
The driver's architecture is heavily inspired by the Linux and U-Boot MMC subsystems, which separate the platform-agnostic SD/MMC protocol logic from the platform-specific host controller driver (HAL). This separation significantly simplifies porting the driver to new hardware, as developers only need to implement the `SdmmcHardware` trait.

Unlike traditional Linux drivers that are tightly coupled with kernel APIs, this driver achieves OS-agnosticism through dependency injection. All OS-dependent services are passed into the driver's initialization function as trait objects. This design ensures maximum portability and makes the driver suitable for a wide range of operating systems and bare-metal applications.

---
*This README was authored by the project maintainer and refined for clarity with assistance from Google's Gemini. All technical information was written and verified by the author.*
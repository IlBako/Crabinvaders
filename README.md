<div align="center">

# 🦀 Crabinvaders

**Space Invaders arcade emulator in Rust, with a hardware-agnostic Intel 8080 instruction core.**

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![License: AGPL v3](https://img.shields.io/github/license/IlBako/Crabinvaders?style=for-the-badge)

</div>

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Demo](#demo)
- [Dependencies](#dependencies)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Overview

Crabinvaders is a from-scratch Space Invaders arcade emulator written in Rust. It's built around a hardware-agnostic Intel 8080 CPU core: the instruction set, registers, and condition flags are implemented independently of the arcade hardware, connecting to memory, video, audio, and I/O only through a generic `Bus`/`IOHandler` abstraction. The CPU's correctness is validated against classic 8080 diagnostic test ROMs (cpudiag, TST8080, 8080PRE, CPUTEST, 8080EXM).

Alongside the emulator itself (`crabinvaders` binary), the project ships a small standalone `disassembler` binary that dumps a disassembly of a ROM to a text file.

Note: the original Space Invaders ROM is not included in this repo (it's copyrighted) — you'll need to supply your own.

## Features

- **Hardware-agnostic Intel 8080 core** — full instruction set, registers, and condition flags, decoupled from arcade-specific hardware via a generic `Bus`/`IOHandler` trait.
- **Space Invaders arcade hardware** — memory-mapped video RAM, DIP-switch-style cabinet settings (starting lives, extra-life threshold, coin-info display) currently fixed at compile time, and full coin/tilt/player 1 & 2 input handling.
- **Cycle-accurate real-time pacing** — the emulation loop throttles itself to match the original board's 1.9968 MHz clock speed.
- **SDL2-based rendering & audio** — OpenGL-backed canvas with a streaming texture, plus SDL2_mixer audio.
- **Standalone disassembler** — a separate `disassembler` binary that dumps a human-readable disassembly of a ROM to a text file.
- **CPU correctness testing** — validated against five classic 8080 diagnostic test ROMs (cpudiag, TST8080, 8080PRE, CPUTEST, 8080EXM).

## Demo

<!-- TODO: add gameplay screenshot(s) or GIF here -->

_Screenshots coming soon._

## Dependencies

Crabinvaders is written in Rust (2024 edition) and built with Cargo. The only external dependency is:

- [`sdl2`](https://crates.io/crates/sdl2) `0.37` (with the `mixer` feature) — handles windowing, OpenGL-backed rendering, keyboard input, and audio mixing.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- SDL2 development libraries, including the `mixer` component

**Linux (Debian/Ubuntu):**
```bash
sudo apt install libsdl2-dev libsdl2-mixer-dev
```

**Windows:**
This project builds against the GNU/MinGW toolchain (`x86_64-pc-windows-gnu`). See the [rust-sdl2 Windows (MinGW) setup guide](https://github.com/Rust-SDL2/rust-sdl2#windows-mingw) for linking SDL2 under MSYS2/MinGW.

### Providing the ROM and sound assets

Crabinvaders does not ship the original Space Invaders ROM or sound files (they're copyrighted). Before building, place your own legally-obtained files at:

```
src/bin/rom/space_invaders/
├── invaders.rom
└── audio/
    ├── 0.wav
    ├── 1.wav
    ├── 2.wav
    ├── 3.wav
    ├── 4.wav
    ├── 5.wav
    ├── 6.wav
    ├── 7.wav
    └── 8.wav
```

> The ROM is currently compiled directly into the binary at build time.

### Build & Run

```bash
git clone https://github.com/IlBako/Crabinvaders.git
cd Crabinvaders
cargo run --release
```

## Usage

Run the emulator:

```bash
cargo run --release
```

### Controls

| Action         | Key           |
|----------------|---------------|
| Insert coin    | `Escape`      |
| Tilt           | `Tab`         |
| Player 1 start | `V`           |
| Player 2 start | `B`           |
| P1 move left   | `Left Arrow`  |
| P1 move right  | `Right Arrow` |
| P1 fire        | `Up Arrow`    |
| P2 move left   | `A`           |
| P2 move right  | `D`           |
| P2 fire        | `W`           |

### Disassembler

The project also includes a standalone disassembler that dumps a human-readable disassembly of the ROM to `out/result.txt`:

```bash
cargo run --bin disassembler
```

## Project Structure

```
Crabinvaders/
├── .cargo/
│   └── config.toml           # Windows (GNU/MinGW) linker config
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
└── src/
    ├── lib.rs
    ├── cpu.rs                 # 8080 CPU: instruction dispatch, generic over IOHandler
    ├── cpu/
    │   ├── bus.rs             # Wires CPU to memory, video, audio, and I/O
    │   ├── instructions.rs    # Opcode implementations
    │   └── registers.rs       # Registers and condition flags
    ├── memory.rs              # Flat 64 KiB address space with optional ROM write-protection
    ├── video.rs               # Video RAM decoding / framebuffer
    ├── audio.rs               # SDL2_mixer sound effect playback
    ├── io.rs                  # IOHandler trait, ArcadeInputs, DipSwitches
    ├── int.rs                 # Interrupt generation
    ├── disassembler.rs        # 8080 instruction decoder; writes disassembly to out/
    ├── utils.rs               # Real-time cycle pacing
    ├── hardware_impl/
    │   ├── mod.rs
    │   └── space_invaders_hw.rs   # Space Invaders-specific IOHandler impl
    ├── tests/
    │   ├── mod.rs
    │   └── test_cpu.rs        # Runs classic 8080 diagnostic test ROMs
    └── bin/
        ├── main.rs             # Emulator entry point
        └── main_disassembler.rs  # Disassembler entry point
```

## Testing

The CPU core has a unit test suite (`src/tests/test_cpu.rs`) that verifies correctness by running it against five classic 8080 diagnostic test ROMs through a small CP/M BDOS-call-intercepting harness:

- `cpudiag.bin`
- `TST8080.COM`
- `8080PRE.COM`
- `CPUTEST.COM`
- `8080EXM.COM`

These test the CPU core in isolation — there is no full integration test suite covering the rest of the emulator (video, audio, arcade I/O) yet.

The ROMs are not included in the repository and must be placed at `src/tests/rom/` before running the suite. Most of them (all except `cpudiag.bin`) can be found at [altairclone.com/downloads/cpu_tests](https://altairclone.com/downloads/cpu_tests/).

```
src/tests/rom/
├── cpudiag.bin
├── TST8080.COM
├── 8080PRE.COM
├── CPUTEST.COM
└── 8080EXM.COM
```

```bash
cargo test
```

> `8080EXM.COM` is the most exhaustive test and can take noticeably longer in debug builds — use `cargo test --release` if it's slow.

## Contributing

Contributions are welcome. To contribute:

1. Fork the repository
2. Create a branch for your change (`git checkout -b feature/your-feature`)
3. Make your changes
4. Commit (`git commit -m 'Add feature'`)
5. Push to your fork (`git push origin feature/your-feature`)
6. Open a pull request

If your change touches CPU behavior, please make sure `cargo test` still passes against the diagnostic ROMs.

## License

This project is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE).

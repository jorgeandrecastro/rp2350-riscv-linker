[![Crates.io](https://img.shields.io/crates/v/rp2350-riscv-linker.svg)](https://crates.io/crates/rp2350-riscv-linker)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![docs.rs](https://docs.rs/rp2350-riscv-linker/badge.svg)](https://docs.rs/rp2350-riscv-linker)

# 🦅 rp2350-riscv-linker

Zero-config linker and boot support for the RISC-V cores of the RP2350 (Raspberry Pi Pico 2) in Rust.

You no longer need to maintain a separate `memory.x`. `rp2350-riscv-linker` generates it at compile time and injects the sections required for RP2350 boot.

This crate provides the RISC-V memory environment for the RP2350 Hazard3 core. It does not replace the application's RISC-V runtime or Embassy crates.

It has been tested with a Raspberry Pi Pico 2 in RP2350 RISC-V mode, together with the `embassy-riscv` fork.

## ✨ Key features

- Automatic `memory.x` injection at compile time
- RISC-V/Hazard3 support for the RP2350
- `.start_block` RP2350 RISC-V image definition for `picotool`
- Default memory layout: 4 MB flash and 520 KB SRAM
- 32-byte RP2350 RISC-V image definition at the start of flash
- Boot entry generated for the application's `_start` reset handler

### Memory map

| Region | Address | Size |
|---|---:|---:|
| Text / flash | `0x10000000` | 3072 KB |
| Read-only data / flash | `0x10300000` | 1024 KB |
| Data / SRAM | `0x20000000` | 64 KB |
| BSS / SRAM | `0x20010000` | 320 KB |
| Heap / SRAM | `0x20060000` | 128 KB |
| Stack / SRAM | `0x20080000` | 8 KB |

## 📦 Installation

> [!IMPORTANT]
> The published `rp2350-riscv-linker` version `0.1` is not compatible with this setup and must not be used. Use version `0.2` instead. `riscv-rt` remains at version `0.18`.

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
rp2350-riscv-linker = "0.2"
riscv-rt = { version = "0.18", optional = true }
```

## 🛠️ Usage

Import the crate at your application's entry point. The build script generates `memory.x` in `OUT_DIR` and adds that directory to the linker search paths.

```rust
#![no_std]
#![no_main]

use rp2350_riscv_linker as _;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

When using `riscv-rt`, enable it from an application feature if needed:

```toml
[features]
default = ["riscv-target"]
riscv-target = ["dep:riscv-rt"]

[dependencies]
riscv-rt = { version = "0.18", optional = true }
```

> [!IMPORTANT]
> The application must provide a RISC-V runtime and startup entry point. `riscv-rt` or an Embassy fork with RISC-V support supplies `link.x`; this crate supplies `memory.x` and the RP2350 boot image definition.

## ⚙️ `.cargo/config.toml` configuration for RISC-V

A working configuration for `riscv32imac-unknown-none-elf` is:

```toml
[build]
target = "riscv32imac-unknown-none-elf"

[target.riscv32imac-unknown-none-elf]
runner = "elf2uf2-rs -d --family rp2350-riscv"

rustflags = [
  "-C", "link-arg=--nmagic",
  "-C", "link-arg=-Tmemory.x",
  "-C", "link-arg=-Tlink.x",
]
```

Additional linker scripts can be added when the runtime or framework requires them:

```toml
rustflags = [
  "-C", "link-arg=--nmagic",
  "-C", "link-arg=-Tmemory.x",
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tembassy_rp_irq.x",
  "-C", "link-arg=-Tdefmt.x",
]
```

## 🧩 Embassy-RISC-V migration

The RP2350 RISC-V migration can use a local `embassy-riscv` fork and this crate together. A package setup that has been tested on Pico 2 is:

```toml
[dependencies]
defmt = "1.0.1"
defmt-rtt = "1.0.0"

embassy-embedded-hal = { path = "../embassy-riscv/embassy-embedded-hal" }

embassy-executor = {
    path = "../embassy-riscv/embassy-executor",
    features = ["executor-thread", "defmt"]
}

embassy-time = {
    path = "../embassy-riscv/embassy-time",
    features = ["defmt", "defmt-timestamp-uptime"]
}

embassy-sync = { path = "../embassy-riscv/embassy-sync" }

embassy-rp = {
    path = "../embassy-riscv/embassy-rp",
    default-features = false,
    features = [
        "rp235xa",
        "riscv",
        "time-driver",
        "critical-section-impl",
        "defmt",
        "imagedef-none",
    ]
}

riscv = "0.11.0"
riscv-rt = { version = "0.18.0", optional = true }
panic-halt = "0.2.0"

rp2350-riscv-linker = { path = "../rp2350-riscv-linker" }

[features]
default = ["riscv-target"]
riscv-target = [
    "dep:riscv-rt",
    "embassy-executor/platform-riscv32",
]

[patch.crates-io]
embassy-time = { path = "../embassy-riscv/embassy-time" }
embassy-time-driver = { path = "../embassy-riscv/embassy-time-driver" }
embassy-time-queue-utils = { path = "../embassy-riscv/embassy-time-queue-utils" }
embassy-sync = { path = "../embassy-riscv/embassy-sync" }
embassy-futures = { path = "../embassy-riscv/embassy-futures" }
embassy-executor = { path = "../embassy-riscv/embassy-executor" }
embassy-executor-timer-queue = { path = "../embassy-riscv/embassy-executor-timer-queue" }
embassy-embedded-hal = { path = "../embassy-riscv/embassy-embedded-hal" }
embassy-rp = { path = "../embassy-riscv/embassy-rp" }
```

The local fork must provide the RISC-V versions of the Embassy crates used by the application. The exact dependency layout can differ between forks and releases.

## 🚀 Flashing workflow

To build, convert the ELF to the RP2350 RISC-V UF2 family, and flash a Pico 2:

```bash
#!/usr/bin/env bash
set -Eeuo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="riscv32imac-unknown-none-elf"
BINARY_NAME="pico2-riscv"
TARGET_PATH="target/$TARGET/release/$BINARY_NAME"
UF2_NAME="pico2_riscv.uf2"

echo "🦅 RP2350 — RISC-V Hazard3"

cargo build --release --target "$TARGET"

if [[ ! -s "$TARGET_PATH" ]]; then
    echo "❌ ELF not found: $TARGET_PATH"
    exit 1
fi

picotool uf2 convert \
    "$TARGET_PATH" \
    -t elf \
    "$UF2_NAME" \
    --family rp2350-riscv

if [[ ! -s "$UF2_NAME" ]]; then
    echo "❌ UF2 generation failed"
    exit 1
fi

sudo picotool load "$UF2_NAME" -x

echo "🚀 Firmware successfully flashed!"
```

Then make it executable and run it:

```bash
chmod +x flash.sh
./flash.sh
```

> [!NOTE]
> This workflow requires `picotool` to be installed and available in your `PATH`.

## 🛡️ License

Licensed under either of:

    MIT license (http://opensource.org/licenses/MIT)

    Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 🦅 About

Developed by Jorge Andre Castro.
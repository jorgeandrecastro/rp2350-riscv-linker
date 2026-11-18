[![Crates.io](https://img.shields.io/crates/v/rp2350-riscv-linker.svg)](https://crates.io/crates/rp2350-riscv-linker)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![docs.rs](https://docs.rs/rp2350-riscv-linker/badge.svg)](https://docs.rs/rp2350-riscv-linker)

# 🦅 rp2350-riscv-linker

Zero-config linker and boot support for the RISC-V cores of the RP2350 (Raspberry Pi Pico 2) in Rust.

You no longer need to maintain a separate `memory.x`. `rp2350-riscv-linker` generates it at compile time and injects the sections required for RP2350 boot.

This crate provides the RISC-V memory environment for the RP2350 Hazard3 core. It does not replace the application's RISC-V runtime.

## ✨ Key features

- Automatic `memory.x` injection at compile time
- RISC-V/Hazard3 support for the RP2350
- `.start_block` and `.boot_info` sections for boot
- `.bi_entries` mapping compatible with `picotool`
- Default memory: 4 MB flash and 512 KB RAM
- Boot sections inserted after `.vector_table`

### Memory map

| Region | Address | Size |
|---|---:|---:|
| FLASH | `0x10000000` | 4096 KB |
| RAM | `0x20000000` | 512 KB |

## 📦 Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
rp2350-riscv-linker = "0.1"
riscv-rt = "0.16"
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

## ⚙️ `.cargo/config.toml` configuration for RISC-V

The RISC-V runtime must provide `link.x` and the `.vector_table` section:

```toml
[target.riscv32imac-unknown-none-elf]
runner = "elf2uf2-rs -d"

rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "link-arg=--nmagic",
]

[build]
target = "riscv32imac-unknown-none-elf"
```

## 🚀 Flashing workflow

To automate build, UF2 conversion, and flashing, create a `flash.sh` script at the root of your project:

```bash
#!/bin/bash
set -e

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="riscv32imac-unknown-none-elf"
FAMILY="rp2350-riscv"

echo "🦅 Target: RP2350 RISC-V"

CRATE_NAME=$(cargo metadata --no-deps --format-version 1 | python3 -c 'import sys, json; print(json.load(sys.stdin)["packages"][0]["name"])')
TARGET_PATH="target/$TARGET/release/$CRATE_NAME"
UF2_NAME="${CRATE_NAME}.uf2"

echo "🦀 Step 1: Compiling release for $TARGET..."
cargo build --release --target "$TARGET" || {
    echo "❌ Compilation failed"
    exit 1
}

echo "📦 Step 2: Converting ELF to UF2 ($FAMILY)..."
picotool uf2 convert -t elf "$TARGET_PATH" "$UF2_NAME" --family "$FAMILY"

echo "⚡ Step 3: Flashing to device..."
sudo picotool load "$UF2_NAME" -x

echo "✅ Done! ${CRATE_NAME} is flying on ${FAMILY}! 🚀"
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
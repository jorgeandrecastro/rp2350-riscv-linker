
use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR").expect("OUT_DIR not defined"),
    );

    let memory_x = r#"
MEMORY
{
    /* RP2350: 4 MiB external QSPI Flash */
    REGION_TEXT   : ORIGIN = 0x10000000, LENGTH = 3072K
    REGION_RODATA : ORIGIN = 0x10300000, LENGTH = 1024K

    /* RP2350: 520 KiB SRAM */
    REGION_DATA   : ORIGIN = 0x20000000, LENGTH = 64K
    REGION_BSS    : ORIGIN = 0x20010000, LENGTH = 320K
    REGION_HEAP   : ORIGIN = 0x20060000, LENGTH = 128K
    REGION_STACK  : ORIGIN = 0x20080000, LENGTH = 8K
}

/*
 * IMAGE_DEF occupies 32 bytes at the beginning of Flash.
 * riscv-rt must therefore start immediately after it.
 */
_stext = ORIGIN(REGION_TEXT) + 32;

/* The RISC-V Boot ROM must enter through the real reset handler. */
ENTRY(_start)

SECTIONS
{
    /*
     * RP2350 PICOBIN IMAGE_DEF for RISC-V.
     *
     * IMAGE_TYPE:
     *   EXE + RP2350 + RISC-V + Security Unspecified
     *
     * ENTRY_POINT:
     *   _start
     *   _stack_start
     */

    .start_block ORIGIN(REGION_TEXT) :
    {
        __start_block_addr = .;

        LONG(0xffffded3)       /* PICOBIN_BLOCK_MARKER_START */
        LONG(0x11010142)       /* RISC-V executable, RP2350 */

        LONG(0x00000344)       /* ENTRY_POINT item */
        LONG(ABSOLUTE(_start))
        LONG(ABSOLUTE(_stack_start))

        LONG(0x000004ff)       /* LAST item, 4 words */
        LONG(0x00000000)       /* Block loop: self */

        LONG(0xab123579)       /* PICOBIN_BLOCK_MARKER_END */

        __start_block_end = .;
    } > REGION_TEXT
}
INSERT BEFORE .text.dummy;

/* Link-time validation */
ASSERT(
    ADDR(.start_block) == ORIGIN(REGION_TEXT),
    "IMAGE_DEF must start at the beginning of Flash"
);

ASSERT(
    SIZEOF(.start_block) == 32,
    "Invalid RP2350 RISC-V IMAGE_DEF size"
);

ASSERT(
    SIZEOF(.start_block) <= 4096,
    "IMAGE_DEF is outside the first 4 KiB"
);

ASSERT(
    _start == _stext,
    "RISC-V reset handler is not at the beginning of .text"
);
"#;

    fs::write(out_dir.join("memory.x"), memory_x)
        .expect("failed to write memory.x");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
}

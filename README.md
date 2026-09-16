# gba_emulator

An early-stage emulator project written in Rust with no external dependencies. The executable loads a ROM and displays its cartridge metadata. The library also provides an initial memory bus.

Despite the package name, the current parser targets the original Game Boy (DMG) cartridge format. Game Boy Advance ROMs are not supported.

## Current Status

- Loads a ROM from a command-line path.
- Displays the title, cartridge type, and declared ROM/RAM sizes.
- Supports ROM ONLY cartridges (`0x00`) with exactly 32 KiB of ROM and no external RAM.
- Reports missing or unreadable files, truncated headers, unknown size codes, incompatible layouts, and ROM size mismatches.
- Explicitly rejects other cartridge types, including those requiring bank switching.

The executable currently inspects metadata only. CPU execution, graphics, audio, input, and save support are not implemented. Nintendo logo and checksum validation are also deferred.

## Getting Started

Install Rust and Cargo with a toolchain that supports the Rust 2024 edition. Run commands from the repository root.

```sh
cargo build
cargo run -- /path/to/game.gb
```

Quote paths containing spaces:

```sh
cargo run -- "/path/to/My Game.gb"
```

Example output for a supported ROM titled `TEST`:

```text
Title: TEST
Cartridge type: ROM ONLY (0x00)
Declared ROM size: 32768 bytes (32 KiB)
Declared RAM size: 0 bytes (0 KiB)
```

The executable requires exactly one ROM path. Errors are printed to standard error and return a nonzero exit status. ROM files are not included.

## Project Structure

```text
src/
├── main.rs       # Command-line arguments, file loading, metadata output
├── lib.rs        # Public library modules
├── bus.rs        # Memory routing, work RAM, high RAM, and bus tests
└── cartridge.rs  # Header parsing, ROM storage, errors, and unit tests
```

File loading stays in the executable. The library accepts owned bytes through `Cartridge::from_bytes(Vec<u8>)`, allowing tests and other callers to construct cartridges without filesystem access. Use `header()` to inspect metadata and `rom()` to access the stored bytes.

## Memory Bus

Construct `Bus::new(cartridge)` using `gba_emulator::bus::Bus`, then call
`read(address)` or `write(address, value)` with 16-bit addresses and byte values.

| Addresses | Current behavior |
| --- | --- |
| `0000–7FFF` | Cartridge ROM reads; writes ignored |
| `8000–9FFF` | VRAM placeholder |
| `A000–BFFF` | No external RAM on ROM ONLY: reads `0xFF`, writes ignored |
| `C000–DFFF` | 8 KiB work RAM |
| `E000–FDFF` | Mirror of work RAM at `C000–DDFF` |
| `FE00–FE9F` | OAM placeholder |
| `FEA0–FEFF` | Unusable memory placeholder |
| `FF00–FF7F` | I/O register placeholders |
| `FF80–FFFE` | 127 bytes of high RAM |
| `FFFF` | Interrupt enable register placeholder |

RAM uses ordinary byte arrays initialized to zero for deterministic development.
This does not model hardware power-on contents. Unfinished regions return
`UNIMPLEMENTED_READ_VALUE` (`0xFF`) and discard writes; every access logs the
region, address, and returned or written value to stderr, including release
builds. These stubs do not model device or open-bus behavior. Replace their
explicit match arms in `src/bus.rs` as devices are implemented. There is no boot
ROM overlay yet.

The address layout follows [Pan Docs](https://gbdev.io/pandocs/Memory_Map.html).

## Development

```sh
cargo test                              # Run tests built from synthetic ROM bytes
cargo fmt --check                       # Check formatting
cargo clippy --all-targets -- -D warnings # Check lints
```

Use `cargo fmt` to apply standard Rust formatting. Tests cover metadata extraction, title handling, truncated headers, unsupported cartridge types, invalid layouts, and incorrect image sizes.

Keep contributions focused, add tests for new parsing behavior, and include the commands used to validate changes in pull requests.

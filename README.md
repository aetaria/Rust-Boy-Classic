# Rust Boy Classic

An early-stage emulator project written in Rust with no external dependencies. The current milestone loads a ROM and displays its cartridge metadata.

Rust Boy Classic targets the original Game Boy (DMG) cartridge format (`.gb`). Game Boy Advance ROMs are not supported. The Rust package, library crate, and executable are named `rust_boy_classic`.

## Current Status

- Loads a ROM from a command-line path.
- Displays the title, cartridge type, and declared ROM/RAM sizes.
- Supports ROM ONLY cartridges (`0x00`) with exactly 32 KiB of ROM and no external RAM.
- Reports missing or unreadable files, truncated headers, unknown size codes, incompatible layouts, and ROM size mismatches.
- Explicitly rejects other cartridge types, including those requiring bank switching.

This milestone inspects metadata only. CPU execution, graphics, audio, input, and save support are not implemented. Nintendo logo and checksum validation are also deferred.

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
└── cartridge.rs  # Header parsing, ROM storage, errors, and unit tests
```

File loading stays in the executable. The library accepts owned bytes through `Cartridge::from_bytes(Vec<u8>)`, allowing tests and other callers to construct cartridges without filesystem access. Use `header()` to inspect metadata and `rom()` to access the stored bytes.

## Development

```sh
cargo test                              # Run tests built from synthetic ROM bytes
cargo fmt --check                       # Check formatting
cargo clippy --all-targets -- -D warnings # Check lints
```

Use `cargo fmt` to apply standard Rust formatting. Tests cover metadata extraction, title handling, truncated headers, unsupported cartridge types, invalid layouts, and incorrect image sizes.

Keep contributions focused, add tests for new parsing behavior, and include the commands used to validate changes in pull requests.

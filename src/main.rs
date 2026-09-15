use std::{env, fs, path::PathBuf, process::ExitCode};

use rust_boy_classic::cartridge::Cartridge;

fn run() -> Result<(), String> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_else(|| "rust_boy_classic".into());
    let usage = || format!("Usage: {} <rom-path>", PathBuf::from(&program).display());
    let path = PathBuf::from(args.next().ok_or_else(usage)?);
    if args.next().is_some() {
        return Err(usage());
    }
    let bytes = fs::read(&path)
        .map_err(|error| format!("could not read ROM '{}': {error}", path.display()))?;
    let cartridge = Cartridge::from_bytes(bytes)
        .map_err(|error| format!("invalid ROM '{}': {error}", path.display()))?;
    let header = cartridge.header();
    println!(
        "Title: {}",
        if header.title.is_empty() {
            "(untitled)"
        } else {
            &header.title
        }
    );
    println!("Cartridge type: {}", header.cartridge_type);
    println!(
        "Declared ROM size: {} bytes ({} KiB)",
        header.rom_size_bytes,
        header.rom_size_bytes / 1024
    );
    println!(
        "Declared RAM size: {} bytes ({} KiB)",
        header.ram_size_bytes,
        header.ram_size_bytes / 1024
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

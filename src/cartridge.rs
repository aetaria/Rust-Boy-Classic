//! Byte-level parsing for original Game Boy cartridge headers.
//!
//! Header layout: https://gbdev.io/pandocs/The_Cartridge_Header.html

use std::{error::Error, fmt};

const HEADER_LEN: usize = 0x150;
const ROM_ONLY_SIZE: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    RomOnly,
}

impl fmt::Display for CartridgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RomOnly => f.write_str("ROM ONLY (0x00)"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub title: String,
    pub cartridge_type: CartridgeType,
    pub rom_size_bytes: usize,
    pub ram_size_bytes: usize,
}

#[derive(Debug)]
pub struct Cartridge {
    header: Header,
    rom: Vec<u8>,
}

impl Cartridge {
    /// Parse owned ROM bytes without performing file I/O.
    ///
    /// Only 32 KiB ROM ONLY cartridges with no external RAM are supported.
    /// The image must match its declared size. Logo and checksum validation
    /// are intentionally deferred, so synthetic test ROMs are accepted.
    pub fn from_bytes(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        if rom.len() < HEADER_LEN {
            return Err(CartridgeError::TruncatedHeader { actual: rom.len() });
        }
        if rom[0x147] != 0 {
            return Err(CartridgeError::UnsupportedType(rom[0x147]));
        }
        let rom_size_bytes = match rom[0x148] {
            code @ 0..=8 => ROM_ONLY_SIZE << code,
            0x52 => 72 * 16 * 1024,
            0x53 => 80 * 16 * 1024,
            0x54 => 96 * 16 * 1024,
            code => return Err(CartridgeError::InvalidRomSize(code)),
        };
        let ram_size_bytes = match rom[0x149] {
            0 => 0,
            // Legacy homebrew tools used this otherwise unused code for 2 KiB.
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            code => return Err(CartridgeError::InvalidRamSize(code)),
        };
        if rom_size_bytes != ROM_ONLY_SIZE || ram_size_bytes != 0 {
            return Err(CartridgeError::UnsupportedLayout {
                rom_size_bytes,
                ram_size_bytes,
            });
        }
        if rom.len() != rom_size_bytes {
            return Err(CartridgeError::RomSizeMismatch {
                declared: rom_size_bytes,
                actual: rom.len(),
            });
        }
        // Color-capable headers reuse the last title byte as a flag.
        let title_end = if rom[0x143] & 0x80 != 0 { 0x143 } else { 0x144 };
        let title_bytes = &rom[0x134..title_end];
        let title_len = title_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(title_bytes.len());
        // Replace malformed/non-printing bytes so metadata is safe to display.
        let title = title_bytes[..title_len]
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    char::from(b)
                } else {
                    '\u{fffd}'
                }
            })
            .collect();
        Ok(Self {
            header: Header {
                title,
                cartridge_type: CartridgeType::RomOnly,
                rom_size_bytes,
                ram_size_bytes,
            },
            rom,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn rom(&self) -> &[u8] {
        &self.rom
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartridgeError {
    TruncatedHeader {
        actual: usize,
    },
    UnsupportedType(u8),
    InvalidRomSize(u8),
    InvalidRamSize(u8),
    UnsupportedLayout {
        rom_size_bytes: usize,
        ram_size_bytes: usize,
    },
    RomSizeMismatch {
        declared: usize,
        actual: usize,
    },
}

impl fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { actual } => write!(
                f,
                "ROM header is truncated: need at least {HEADER_LEN} bytes, got {actual}"
            ),
            Self::UnsupportedType(code) => write!(
                f,
                "unsupported cartridge type 0x{code:02X}; only ROM ONLY (0x00) is supported"
            ),
            Self::InvalidRomSize(code) => write!(f, "unknown ROM size code 0x{code:02X}"),
            Self::InvalidRamSize(code) => write!(f, "unknown RAM size code 0x{code:02X}"),
            Self::UnsupportedLayout {
                rom_size_bytes,
                ram_size_bytes,
            } => write!(
                f,
                "unsupported ROM ONLY layout: declares {rom_size_bytes} ROM bytes and {ram_size_bytes} RAM bytes; expected 32768 ROM bytes and no RAM"
            ),
            Self::RomSizeMismatch { declared, actual } => write!(
                f,
                "ROM size mismatch: header declares {declared} bytes, file contains {actual}"
            ),
        }
    }
}

impl Error for CartridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom() -> Vec<u8> {
        let mut bytes = vec![0; ROM_ONLY_SIZE];
        bytes[0x134..0x138].copy_from_slice(b"TEST");
        bytes
    }

    #[test]
    fn parses_rom_only_metadata_and_preserves_bytes() {
        let bytes = rom();
        let cartridge = Cartridge::from_bytes(bytes.clone()).unwrap();
        assert_eq!(
            cartridge.header(),
            &Header {
                title: "TEST".into(),
                cartridge_type: CartridgeType::RomOnly,
                rom_size_bytes: 32768,
                ram_size_bytes: 0,
            }
        );
        assert_eq!(cartridge.rom(), bytes);
    }

    #[test]
    fn rejects_every_truncated_header_length() {
        for actual in 0..HEADER_LEN {
            assert_eq!(
                Cartridge::from_bytes(vec![0; actual]).unwrap_err(),
                CartridgeError::TruncatedHeader { actual }
            );
        }
    }

    #[test]
    fn rejects_all_other_cartridge_types() {
        for code in 1..=255 {
            let mut bytes = rom();
            bytes[0x147] = code;
            assert_eq!(
                Cartridge::from_bytes(bytes).unwrap_err(),
                CartridgeError::UnsupportedType(code)
            );
        }
    }

    #[test]
    fn rejects_short_and_oversized_images() {
        for actual in [HEADER_LEN, ROM_ONLY_SIZE - 1, ROM_ONLY_SIZE + 1] {
            let mut bytes = rom();
            bytes.resize(actual, 0);
            assert_eq!(
                Cartridge::from_bytes(bytes).unwrap_err(),
                CartridgeError::RomSizeMismatch {
                    declared: ROM_ONLY_SIZE,
                    actual
                }
            );
        }
    }

    #[test]
    fn rejects_unknown_size_codes_and_incompatible_layouts() {
        for (offset, code, expected) in [
            (0x148, 0xff, CartridgeError::InvalidRomSize(0xff)),
            (0x149, 0xff, CartridgeError::InvalidRamSize(0xff)),
            (
                0x148,
                1,
                CartridgeError::UnsupportedLayout {
                    rom_size_bytes: 65536,
                    ram_size_bytes: 0,
                },
            ),
            (
                0x149,
                2,
                CartridgeError::UnsupportedLayout {
                    rom_size_bytes: 32768,
                    ram_size_bytes: 8192,
                },
            ),
        ] {
            let mut bytes = rom();
            bytes[offset] = code;
            assert_eq!(Cartridge::from_bytes(bytes).unwrap_err(), expected);
        }
    }

    #[test]
    fn handles_full_titles_color_flags_and_non_printing_bytes() {
        let mut bytes = rom();
        bytes[0x134..0x144].copy_from_slice(b"ABCDEFGHIJKLMNOP");
        assert_eq!(
            Cartridge::from_bytes(bytes.clone()).unwrap().header().title,
            "ABCDEFGHIJKLMNOP"
        );
        bytes[0x143] = 0x80;
        assert_eq!(
            Cartridge::from_bytes(bytes.clone()).unwrap().header().title,
            "ABCDEFGHIJKLMNO"
        );
        bytes[0x134..0x138].copy_from_slice(&[b'A', 0x1b, 0xff, 0]);
        assert_eq!(
            Cartridge::from_bytes(bytes).unwrap().header().title,
            "A\u{fffd}\u{fffd}"
        );
    }
}

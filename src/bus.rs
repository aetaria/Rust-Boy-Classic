//! Initial DMG memory bus. No boot ROM overlay or device timing yet.
//! Map: https://gbdev.io/pandocs/Memory_Map.html

use crate::cartridge::Cartridge;

/// Temporary value for unfinished devices, not an emulation of open-bus behavior.
pub const UNIMPLEMENTED_READ_VALUE: u8 = 0xFF;

#[derive(Debug)]
pub struct Bus {
    cartridge: Cartridge,
    work_ram: [u8; 0x2000],
    high_ram: [u8; 0x7F],
}

impl Bus {
    /// RAM starts at zero for deterministic development; hardware startup
    /// contents are not modeled yet.
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            work_ram: [0; 0x2000],
            high_ram: [0; 0x7F],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read(address),
            0xC000..=0xDFFF => self.work_ram[usize::from(address - 0xC000)],
            // Echo maps only C000..=DDFF, not the final 512 bytes of WRAM.
            0xE000..=0xFDFF => self.work_ram[usize::from(address - 0xE000)],
            0xFF80..=0xFFFE => self.high_ram[usize::from(address - 0xFF80)],
            0x8000..=0x9FFF => unfinished_read("VRAM", address),
            0xFE00..=0xFE9F => unfinished_read("OAM", address),
            0xFEA0..=0xFEFF => unfinished_read("unusable memory", address),
            0xFF00..=0xFF7F => unfinished_read("I/O registers", address),
            0xFFFF => unfinished_read("interrupt enable register", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write(address, value),
            0xC000..=0xDFFF => self.work_ram[usize::from(address - 0xC000)] = value,
            0xE000..=0xFDFF => self.work_ram[usize::from(address - 0xE000)] = value,
            0xFF80..=0xFFFE => self.high_ram[usize::from(address - 0xFF80)] = value,
            0x8000..=0x9FFF => unfinished_write("VRAM", address, value),
            0xFE00..=0xFE9F => unfinished_write("OAM", address, value),
            0xFEA0..=0xFEFF => unfinished_write("unusable memory", address, value),
            0xFF00..=0xFF7F => unfinished_write("I/O registers", address, value),
            0xFFFF => unfinished_write("interrupt enable register", address, value),
        }
    }
}

// Keep temporary device behavior in these helpers and explicit match arms so
// each device can replace its stub without changing the supported RAM paths.
fn unfinished_read(region: &str, address: u16) -> u8 {
    eprintln!(
        "[bus: unfinished {region}] read 0x{address:04X} -> placeholder 0x{UNIMPLEMENTED_READ_VALUE:02X}"
    );
    UNIMPLEMENTED_READ_VALUE
}

fn unfinished_write(region: &str, address: u16, value: u8) {
    eprintln!("[bus: unfinished {region}] ignored write 0x{value:02X} to 0x{address:04X}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bus() -> Bus {
        let mut rom = vec![0; 0x8000];
        for address in [0, 0x2000, 0x3FFF, 0x4000, 0x6000, 0x7FFF] {
            rom[address] = (address / 256) as u8 + 1;
        }
        Bus::new(Cartridge::from_bytes(rom).unwrap())
    }

    #[test]
    fn forwards_cartridge_reads_and_ignores_rom_writes() {
        let mut bus = bus();
        for address in [0, 0x2000, 0x3FFF, 0x4000, 0x6000, 0x7FFF] {
            let expected = (address / 256) as u8 + 1;
            assert_eq!(bus.read(address), expected);
            bus.write(address, 0xEE);
            assert_eq!(bus.read(address), expected);
        }
        for address in [0xA000, 0xB000, 0xBFFF] {
            bus.write(address, 0x42);
            assert_eq!(bus.read(address), 0xFF);
        }
    }

    #[test]
    fn ram_boundaries_and_middle_store_independent_bytes() {
        let mut bus = bus();
        let addresses = [
            0xC000, 0xC800, 0xCFFF, 0xD000, 0xDDFF, 0xDE00, 0xDFFF, 0xFF80, 0xFFBF, 0xFFFE,
        ];
        for (i, &address) in addresses.iter().enumerate() {
            assert_eq!(bus.read(address), 0);
            bus.write(address, i as u8 + 1);
        }
        for (i, address) in addresses.into_iter().enumerate() {
            assert_eq!(bus.read(address), i as u8 + 1);
            bus.write(address, 0xFF);
            assert_eq!(bus.read(address), 0xFF);
            bus.write(address, 0);
            assert_eq!(bus.read(address), 0);
        }
    }

    #[test]
    fn echo_ram_mirrors_work_ram_in_both_directions() {
        let mut bus = bus();
        for address in [0xC000, 0xC800, 0xCFFF, 0xD000, 0xDDFF] {
            bus.write(address, 0x12);
            assert_eq!(bus.read(address + 0x2000), 0x12);
            bus.write(address + 0x2000, 0x34);
            assert_eq!(bus.read(address), 0x34);
        }
    }

    #[test]
    fn unfinished_regions_use_placeholder_without_corrupting_ram() {
        let mut bus = bus();
        for address in [0xDE00, 0xDFFF, 0xFF80, 0xFFFE] {
            bus.write(address, 0x42);
        }
        for address in [
            0x8000, 0x9000, 0x9FFF, 0xFE00, 0xFE50, 0xFE9F, 0xFEA0, 0xFEC0, 0xFEFF, 0xFF00, 0xFF40,
            0xFF7F, 0xFFFF,
        ] {
            assert_eq!(bus.read(address), UNIMPLEMENTED_READ_VALUE);
            bus.write(address, 0xAB);
            assert_eq!(bus.read(address), UNIMPLEMENTED_READ_VALUE);
        }
        for address in [0xDE00, 0xDFFF, 0xFF80, 0xFFFE] {
            assert_eq!(bus.read(address), 0x42);
        }
    }
}

use rom::{Cartridge};
use ppu::{PpuInterface};

pub struct Memory {
    ram: [u8; 0x2000],
    cartridge: Cartridge,
    ppu_interface: PpuInterface,
}

impl Memory {
    pub fn new(cartridge: Cartridge) -> Memory {
        Memory {
            ram: [0; 0x2000],
            cartridge: cartridge,
            ppu_interface: PpuInterface::new(),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // RAM
            0x0000 ... 0x1fff => {
                // RAM mirrored from 0x0800 to 0x2000.
                let offset = addr % 0x0800;
                self.ram[offset as usize]
            },
            // PPU Registers
            0x2000 ... 0x3fff | 0x4014 => {
                self.ppu_interface.read(addr)
            },
            // I/O Registers
            0x4000 ... 0x4013 | 0x4015 ... 0x4017 => {
                panic!("Non-PPU I/O reads not implemented: {:#x}", addr)
            },
            // Expansion ROM
            0x4020 ... 0x5fff => {
                panic!("Expansion ROM reads not implemented: {:#x}", addr)
            },
            0x6000 ... 0xffff => {
                self.cartridge.read(addr)
            },
            _ => panic!("Invalid write to memory at: {:#x}", addr),
        }
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        (lo as u16) | ((hi as u16) << 8)
    }

    pub fn write(&mut self, addr: u16, x: u8) {
        match addr {
            // RAM
            0x0000 ... 0x1fff => {
                let offset = addr % 0x0800;
                self.ram[offset as usize] = x;
            },
            // PPU Registers
            0x2000 ... 0x3fff | 0x4014 => {
                self.ppu_interface.write(addr, x);
            },
            // I/O Registers
            0x4000 ... 0x4013 | 0x4015 ... 0x4017 => {
                panic!("Non-PPU I/O writes not implemented: {:#x}", addr)
            },
            // Expansion ROM
            0x4020 ... 0x5fff => {
                panic!("Expansion ROM writes not implemented")
            },
            0x6000 ... 0xffff => {
                self.cartridge.write(addr, x);
            },
            _ => unreachable!(),
        }
    }
}
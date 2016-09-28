use rom::{Cartridge};
use ppu::{PpuInterface};

pub struct Interconnect {
    ram: Vec<u8>,
    cartridge: Cartridge,
    ppu_interface: PpuInterface,
    dma: bool,
}

impl Interconnect {
    pub fn new(cartridge: Cartridge) -> Self {
        Interconnect {
            ram: vec![0; 0x2000],
            cartridge: cartridge,
            ppu_interface: PpuInterface::new(),
            dma: false,
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // RAM
            0x0000 ... 0x1fff => {
                // RAM mirrored from 0x0800 to 0x2000.
                let offset = addr % 0x0800;
                self.ram[offset as usize]
            },
            // PPU Registers
            0x2000 ... 0x3fff => {
                self.ppu_interface.read_register(addr)
            },
            0x4014 => {
                panic!("Cannot read from write-only PPU DMA register")
            },
            // I/O Registers
            0x4000 ... 0x4013 | 0x4015 ... 0x4017 => {
                panic!("I/O reads not implemented: {:#x}", addr)
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

    pub fn cpu_read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.cpu_read(addr);
        let hi = self.cpu_read(addr + 1);
        (lo as u16) | ((hi as u16) << 8)
    }

    pub fn cpu_write(&mut self, addr: u16, x: u8) {
        match addr {
            // RAM
            0x0000 ... 0x1fff => {
                let offset = addr % 0x0800;
                self.ram[offset as usize] = x;
            },
            // PPU Registers
            0x2000 ... 0x3fff => {
                self.ppu_interface.write_register(addr, x);
            },
            0x4014 => {
                // Perform Sprite DMA process.
                let dma_addr = (x * 0x100) as u16;
                for i in 0..256 {
                    let value = self.cpu_read(dma_addr + i);
                    self.ppu_interface.write_spr(i as u8, value);
                }
            },
            // I/O Registers
            0x4000 ... 0x4013 | 0x4015 ... 0x4017 => {
                panic!("I/O reads not implemented: {:#x}", addr)
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

    pub fn ppu_read(&self, addr: u16) -> u8 {
        self.ppu_interface.read(addr)
    }

    pub fn dma(&self) -> bool { self.dma }

    pub fn set_dma(&mut self, value: bool) { self.dma = value; }
}


//struct CpuInterface {
//
//}
//
//impl CpuInterface {
//    pub fn read(&self, addr: u16) -> u8 {
//
//    }
//
//    pub fn write(&mut self, addr: u16, x: u8) {
//
//    }
//}
//
//struct PpuInterface {
//}
//
//impl PpuInterface {
//    pub fn read(&self, addr: u16) -> u8 {
//
//    }
//
//    pub fn write(&mut self, addr: u16, x: u8) {
//
//    }
//}
//
//interconnect.cpu().read(0x2000);
//interconnect.cpu().write(0x2002, 0x10);
//interconnect.ppu().read(0x8000);
//interconnect.ppu().write(0x1000, 0x20);
//let cpu_cycles = if interconnect.dma() {
//    1
//} else {
//    cpu.step(&mut interconnect)
//};
//
//ppu.step(&mut interconnect);
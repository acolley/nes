use rom::{Cartridge};

pub struct NesMemory {
    // ram: [u8; 0x0800],
    ram: [u8; 0xffff],
}

impl NesMemory {
    pub fn new() -> NesMemory {
        NesMemory {
            // ram: [0; 0x0800],
            ram: [0; 0xffff],
        }
    }
}

impl NesMemory {
    pub fn read(&self, addr: u16) -> u8 {
        // let addr = NesAddr::new(addr);
        // match addr {
        //     NesAddr::Ram(i) => self.ram[i],
        // }
        self.ram[addr as usize]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        // match NesAddr::new(addr) {
        //     NesAddr::Ram(i) => {
        //         let lo = self.ram[i];
        //         let hi = self.ram[i + 1];
        //         (lo as u16) | ((hi as u16) << 8)
        //     }
        // }
        let lo = self.ram[addr as usize];
        let hi = self.ram[(addr + 1) as usize];
        (lo as u16) | ((hi as u16) << 8)
    }

    pub fn write(&mut self, addr: u16, x: u8) {
        // let addr = NesAddr::new(addr);
        // match addr {
        //     NesAddr::Ram(i) => self.ram[i] = x,
        // }
        self.ram[addr as usize] = x;
    }
}

pub struct Memory {
    ram: [u8; 0x2000],
    cartridge: Cartridge,
}

impl Memory {
    pub fn new(cartridge: Cartridge) -> Memory {
        Memory {
            ram: [0; 0x2000],
            cartridge: cartridge,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // RAM
            0x0000 ... 0x1fff => {
                // RAM mirrored from 0x0800 to 0x2000.
                let offset = addr % 0x0800;
                self.ram[offset as usize]
            },
            // I/O Registers
            0x2000 ... 0x401f => {
                panic!("I/O reads not implemented")
            },
            // Expansion ROM
            0x4020 ... 0x5fff => {
                panic!("Expansion ROM reads not implemented")
            },
            0x6000 ... 0xffff => {
                self.cartridge.read(addr)
            },
            _ => unreachable!(),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
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
            // I/O Registers
            0x2000 ... 0x401f => {
                panic!("I/O writes not implemented")
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
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
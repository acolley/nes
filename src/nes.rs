use cpu::Cpu;
use memory::NesMemory;

pub struct Nes {
    mem: NesMemory,
    cpu: Cpu,
}

impl Nes {
    pub fn new(rom: Vec<u8>) -> Nes {
        let mut memory = NesMemory::new();
        for (i, x) in rom.iter().enumerate() {
            memory.write(i as u16, x.clone());
        }

        Nes {
            mem: NesMemory::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.mem);
    }
}

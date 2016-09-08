use cpu::Cpu;
use memory::NesMemory;

pub struct Nes {
    mem: NesMemory,
    cpu: Cpu,
}

impl Nes {
    pub fn new(rom: Vec<u8>) -> Nes {
        Nes {
            mem: NesMemory::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.mem);
    }
}

use cpu::Cpu;
use memory::Memory;
use rom::Cartridge;

pub struct Nes {
    cpu: Cpu,
    mem: Memory,
}

impl Nes {
    pub fn new(cartridge: Cartridge) -> Nes {
        let mem = Memory::new(cartridge);
        let mut cpu = Cpu::new();
        cpu.reset(&mem);
        Nes {
            cpu: cpu,
            mem: mem,
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn mem(&self) -> &Memory {
        &self.mem
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.mem);
    }
}

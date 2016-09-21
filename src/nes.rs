use cpu::Cpu;
use interconnect::Interconnect;
use rom::Cartridge;

pub struct Nes {
    cpu: Cpu,
    interconnect: Interconnect,
}

impl Nes {
    pub fn new(cartridge: Cartridge) -> Nes {
        let mut interconnect = Interconnect::new(cartridge);
        let mut cpu = Cpu::new();
        cpu.reset(&mut interconnect);
        Nes {
            cpu: cpu,
            interconnect: interconnect,
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn interconnect(&self) -> &Interconnect {
        &self.interconnect
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.interconnect);
    }
}

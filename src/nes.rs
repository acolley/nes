use cpu::{Cpu, Instruction};
use interconnect::Interconnect;
use ppu::Ppu;
use rom::Cartridge;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    interconnect: Interconnect,
}

impl Nes {
    pub fn new(cartridge: Cartridge) -> Nes {
        let mut interconnect = Interconnect::new(cartridge);
        let mut cpu = Cpu::new();
        cpu.reset(&mut interconnect);
        Nes {
            cpu: cpu,
            ppu: Ppu::new(),
            interconnect: interconnect,
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn interconnect(&mut self) -> &mut Interconnect {
        &mut self.interconnect
    }

    pub fn peek(&mut self) -> u8 {
        self.cpu.peek(&mut self.interconnect)
    }

    pub fn peek_u16(&mut self) -> u16 {
        self.cpu.peek_u16(&mut self.interconnect)
    }

    pub fn skip_peek(&mut self, skip: usize) -> u8 {
        self.cpu.skip_peek(skip, &mut self.interconnect)
    }

    pub fn skip_peek_u16(&mut self, skip: usize) -> u16 {
        self.cpu.skip_peek_u16(skip, &mut self.interconnect)
    }

    pub fn current_instruction(&mut self) -> Instruction {
        self.cpu.current_instruction(&mut self.interconnect)
    }

    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.step(&mut self.interconnect);

        // PPU runs 3 cycles per CPU cycle so let it catch up
        for _ in 0..cpu_cycles * 3 {
            self.ppu.step(&mut self.interconnect);
        }
    }

    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
}

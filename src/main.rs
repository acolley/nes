mod cpu;
mod memory;
mod nes;

use cpu::Cpu;
use nes::Nes;

fn main() {
    let rom = Vec::new();
    let mut nes = Nes::new(rom);
    loop {
        nes.step();
    }
}

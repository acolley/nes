mod command;

use std::io::{Write, stdin, stdout};

use super::cpu::{AddressMode};
use super::nes::Nes;
use self::command::Command;

pub struct Debugger {
    nes: Nes,
    last_command: Option<Command>,
}

impl Debugger {
    pub fn new(nes: Nes) -> Debugger {
        Debugger {
            nes: nes,
            last_command: None,
        }
    }

    fn print_instruction(&mut self) {
        let instruction = self.nes.current_instruction();
        let operand = match instruction.address_mode {
            AddressMode::Accumulator | AddressMode::Implied => {
                "".into()
            },
            AddressMode::Absolute => format!("${:04x}", self.nes.skip_peek_u16(1)),
            AddressMode::AbsoluteXIndexed => format!("${:04x},X", self.nes.skip_peek_u16(1)),
            AddressMode::AbsoluteYIndexed => format!("${:04x},Y", self.nes.skip_peek_u16(1)),
            AddressMode::Immediate => format!("#${:02x}", self.nes.skip_peek(1)),
            AddressMode::Relative => format!("${:02x}", self.nes.skip_peek(1)),
            AddressMode::Indirect => format!("(${:04x})", self.nes.skip_peek_u16(1)),
            AddressMode::XIndexedIndirect => format!("(${:02x},X)", self.nes.skip_peek(1)),
            AddressMode::IndirectYIndexed => format!("(${:02x}),Y", self.nes.skip_peek(1)),
            AddressMode::ZeroPage => format!("${:02x}", self.nes.skip_peek(1)),
            AddressMode::ZeroPageXIndexed => format!("${:02x},X", self.nes.skip_peek(1)),
            AddressMode::ZeroPageYIndexed => format!("${:02x},Y", self.nes.skip_peek(1)),
        };
        println!("{:04x} {:?} {}", self.nes.cpu().reg.pc, instruction, operand   );
    }

    pub fn run(&mut self) {
        self.print_instruction();
        print!(">");
        loop {
            stdout().flush().unwrap();

            let command = match (read_stdin().parse(), self.last_command) {
                (Ok(Command::Repeat), Some(c)) => Ok(c),
                (Ok(Command::Repeat), None) => Err("No last command to repeat".into()),
                (Ok(c), _) => Ok(c),
                (Err(e), _) => Err(e),
            };

            match command {
                Ok(Command::Step(count)) => self.step_by(count),
                Ok(Command::Exit) => break,
                Ok(Command::Repeat) => unreachable!(),
                Err(ref e) => println!("{}", e),
            }

            self.last_command = command.ok();

            print!(">");
        }
    }

    pub fn step(&mut self) {
        self.nes.step();

        self.print_instruction();
    }

    pub fn step_by(&mut self, count: usize) {
        for _ in 0..count {
            self.step();
        }
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}
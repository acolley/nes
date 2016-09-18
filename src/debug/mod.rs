mod command;

use std::io::{stdin, stdout};

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

    pub fn run(&mut self) {
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
        let instruction = self.nes.cpu().current_instruction(self.nes.mem());

        println!("{:?}", instruction);

        self.nes.step();
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
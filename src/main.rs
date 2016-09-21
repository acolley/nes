extern crate clap;
#[macro_use]
extern crate nom;

mod cpu;
mod debug;
mod interconnect;
mod nes;
mod ppu;
mod rom;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{Arg, App, SubCommand};

use cpu::Cpu;
use nes::Nes;
use rom::Cartridge;

fn create_console<P: AsRef<Path>>(filename: P) -> Nes {
    let cartridge = match Cartridge::from_file(&filename) {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };
    Nes::new(cartridge)
}

fn main() {
    let opts = App::new("nes")
        .version("0.1")
        .subcommand(SubCommand::with_name("emu")
            .arg(Arg::with_name("FILENAME")
                .required(true)))
        .subcommand(SubCommand::with_name("dbg")
            .arg(Arg::with_name("FILENAME")
                .required(true)))
        .get_matches();

    match opts.subcommand() {
        ("emu", Some(subopts)) => {
            let filename = subopts.value_of("FILENAME").unwrap();
            let mut console = create_console(&filename);
            console.run();
        },
        ("dbg", Some(subopts)) => {
            let filename = subopts.value_of("FILENAME").unwrap();
            let console = create_console(&filename);
            let mut debugger = debug::Debugger::new(console);
            debugger.run();
        },
        _ => unreachable!(),
    }
}

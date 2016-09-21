extern crate clap;
#[macro_use]
extern crate nom;

mod cpu;
mod interconnect;
mod nes;
mod ppu;
mod rom;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{Arg, App};

use cpu::Cpu;
use nes::Nes;
use rom::Cartridge;

fn main() {
    let options = App::new("nes")
        .version("0.1")
        .arg(Arg::with_name("FILENAME")
            .required(true))
        .get_matches();

    let filename = options.value_of("FILENAME").unwrap();

    let cartridge = match Cartridge::from_file(&filename) {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };

    let mut nes = Nes::new(cartridge);
    loop {
        nes.step();
    }
    // let rom = Rom::from_bytes(data).expect("Rom format not recognised");
    // println!("{}", rom.data.len());
    // let mut nes = Nes::new(rom);
    // loop {
    //     nes.step();
    // }
}

extern crate clap;
#[macro_use]
extern crate nom;

mod cpu;
mod memory;
mod nes;
mod rom;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{Arg, App};

use cpu::Cpu;
use nes::Nes;
use rom::Rom;

fn read_file(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).expect("Could not read file");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

fn main() {
    let options = App::new("nes")
        .version("0.1")
        .arg(Arg::with_name("FILENAME")
            .required(true))
        .get_matches();

    let filename = options.value_of("FILENAME").unwrap();
    let data = read_file(&filename);
    let rom = Rom::from_bytes(data).expect("Rom format not recognised");
    // println!("{}", rom.data.len());
    // let mut nes = Nes::new(rom);
    // loop {
    //     nes.step();
    // }
}

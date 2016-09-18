//! Define a structure that can represent
//! a standard ROM binary.
//! Currently only supports iNES ROMs.


// TODO: support Unif ROM format.

use std::borrow::{Cow};
use std::error;
use std::fmt;
use std::fs::{File};
use std::io;
use std::io::{Read};
use std::mem;
use std::path::{Path};
use std::result;
use std::str;

use nom;

use super::mapper::{Mapper, Mapper0, Mapper1};

struct INesHeader {
    pub nprg: u8,
    pub nchr: u8,
    control1: u8,
    control2: u8,
    nram: u8,
}

impl INesHeader {
    pub fn mapper(&self) -> Box<Mapper> {
        match ((self.control1 & 0xf0) >> 4) | (self.control2 & 0xf0) {
            0x00 => Box::new(Mapper0 { nprg: self.nprg as usize }) as Box<Mapper>,
            0x01 => Box::new(Mapper1) as Box<Mapper>,
            n => panic!("Unrecognised mapper: {:#x}", n),
        }
    }

    pub fn has_trainer(&self) -> bool {
        (self.control1 & 0b10) != 0
    }

    pub fn sram(&self) -> Vec<u8> {
        if self.nram == 0 {
            vec![0; 8192]
        } else {
            vec![0; 8192 * (self.nram as usize)]
        }
    }
}

pub struct Cartridge {
    sram: Vec<u8>, // Save RAM (i.e. PRG RAM)
    prg: Vec<u8>,
    chr: Vec<u8>,
    mapper: Box<Mapper>,
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cartridge {{ sram: {}, prg: {}, chr: {} }}", 
            self.sram.len(), self.prg.len(), self.chr.len())
    }
}

// Parse the 16 byte header
named!(parse_header<INesHeader>,
    chain!(
        tag!( "NES" ) ~
        tag!( &[0x1a] ) ~

        nprg: call!( nom::le_u8 ) ~
        nchr: call!( nom::le_u8 ) ~
        control1: call!( nom::le_u8 ) ~
        control2: call!( nom::le_u8 ) ~
        nram: call!( nom::le_u8 ) ~
        count!( call!( nom::le_u8 ), 7 ) ,
        || INesHeader {
            nprg: nprg,
            nchr: nchr,
            control1: control1,
            control2: control2,
            nram: nram,
        }
    )
);

named!(parse_cartridge<Cartridge>,
    complete!(
        chain!(
            header: parse_header ~

            // Skip trainer
            cond!(
                header.has_trainer(),
                take!( 512 )
            ) ~

            prg: count!( 
                call!( nom::le_u8 ), 
                16384 * (header.nprg as usize) 
            ) ~
            chr: count!(
                call!( nom::le_u8 ),
                8192 * (header.nchr as usize)
            ) ,
            || Cartridge {
                sram: header.sram(),
                prg: prg,
                chr: chr,
                mapper: header.mapper(),
            }
        )
    )
);

pub type Result<T> = result::Result<T, Error>;

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Result<Cartridge> {
        // try!(parse_cartridge(&data))
        match parse_cartridge(&data) {
            nom::IResult::Done(_, cartridge) => Ok(cartridge),
            nom::IResult::Error(_) => result::Result::Err(
                Error::Parse("Could not parse ROM, unrecognised format.".into())
            ),
            _ => unreachable!(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Cartridge> {
        let mut file = try!(File::open(path));
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        Cartridge::new(buf)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ... 0x1fff => {
                let offset = self.mapper.map_chr(addr);
                self.chr[offset]
            },
            0x6000 ... 0x7fff => {
                let offset = self.mapper.map_sram(addr);
                self.sram[offset]
            },
            0x8000 ... 0xffff => {
                let offset = self.mapper.map_prg(addr);
                self.prg[offset]
            },
            _ => panic!("Invalid memory access: {:#x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, x: u8) {
        match addr {
            0x0000 ... 0x1fff => {
                let offset = self.mapper.map_chr(addr);
                self.chr[offset] = x;
            },
            0x6000 ... 0x7fff => {
                let offset = self.mapper.map_sram(addr);
                self.sram[offset] = x;
            },
            0x8000 ... 0xffff => {
                let offset = self.mapper.map_prg(addr);
                self.prg[offset] = x;
            },
            _ => panic!("Invalid memory access: {:#x}", addr),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Parse(err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            _ => Some(self)
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

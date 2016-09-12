//! Define a structure that can represent
//! a standard ROM binary.
//! Currently only supports iNES ROMs.


// TODO: support Unif ROM format.

use std::mem;
use std::path::{Path};
use std::str;

use nom;

// pub trait Rom {

// }

pub struct INesRom {
    pub magic: u32,
    pub prg: u8,
    pub chr: u8,
    pub control1: u8,
    pub control2: u8,
    pub ram: u8,
    pub data: Vec<u8>,
}

named!(parse_rom<INesRom>,
    chain!(
        magic: call!(nom::le_u32) ~
        prg: call!(nom::le_u8) ~
        chr: call!(nom::le_u8) ~
        control1: call!(nom::le_u8) ~
        control2: call!(nom::le_u8) ~
        ram: call!(nom::le_u8) ~
        data: many1!(call!(nom::le_u8)) ,
        || INesRom {
            magic: magic,
            prg: prg,
            chr: chr,
            control1: control1,
            control2: control2,
            ram: ram,
            data: data
        }
    )
);

pub struct Rom {
    pub data: Vec<u8>,
}

impl Rom {
    pub fn from_bytes(data: Vec<u8>) -> Option<Rom> {
        let header = parse_rom(&data);
        // Rom { data: &data[mem::size_of::<INesRomHeader>()..].into() }
        match header {
            nom::IResult::Done(_, rom) => Some(Rom {
                data: rom.data
            }),
            _ => None
        }
    }
}
use std::fmt;

use self::AddressMode::*;
use self::Mnemonic::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mnemonic {
    ADC, AND,
    ASL, BCC,
    BCS, BEQ,
    BIT, BMI,
    BNE, BPL,
    BRK, BVC,
    BVS, CLC,
    CLD, CLI,
    CLV, CMP,
    CPX, CPY,
    DEC, DEX,
    DEY, EOR,
    INC, INX,
    INY, JMP,
    JSR, LDA,
    LDX, LDY,
    LSR, NOP,
    ORA, PHA,
    PHP, PLA,
    PLP, ROL,
    ROR, RTI,
    RTS, SBC,
    SEC, SED,
    SEI, STA,
    STX, STY,
    TAX, TAY,
    TSX, TXA,
    TXS, TYA,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressMode {
    Accumulator,
    Absolute,
    AbsoluteXIndexed,
    AbsoluteYIndexed,
    Immediate,
    Implied,
    Indirect,
    XIndexedIndirect,
    IndirectYIndexed,
    Relative,
    ZeroPage,
    ZeroPageXIndexed,
    ZeroPageYIndexed,
}

// 151 Op Codes
// Using the page at: http://www.llx.com/~nparker/a2/opcodes.html.

#[inline(always)]
fn match_cc_00(aaa: u8, bbb: u8) -> (Mnemonic, AddressMode) {
    let mnemonic = match aaa {
        0b001 => BIT,
        0b010 => JMP,
        0b011 => JMP,
        0b100 => STY,
        0b101 => LDY,
        0b110 => CPY,
        0b111 => CPX,
        _ => panic!("Unrecognised op code: {:02x}", (aaa << 5) | (bbb << 2)),
    };
    let address_mode = match bbb {
        0b000 => Immediate,
        0b001 => ZeroPage,
        0b011 => Absolute,
        0b101 => ZeroPageXIndexed,
        0b111 => AbsoluteXIndexed,
        _ => panic!("Unrecognised op code: {:02x}", (aaa << 5) | (bbb << 2)),
    };
    (mnemonic, address_mode)
}

#[inline(always)]
fn match_cc_01(aaa: u8, bbb: u8) -> (Mnemonic, AddressMode) {
    let mnemonic = match aaa {
        0b000 => ORA,
        0b001 => AND,
        0b010 => EOR,
        0b011 => ADC,
        0b100 => STA,
        0b101 => LDA,
        0b110 => CMP,
        0b111 => SBC,
        _ => panic!("Unrecognised op code: {:02x}", (aaa << 5) | (bbb << 2) | 0x01),
    };
    let address_mode = match bbb {
        0b000 => XIndexedIndirect,
        0b001 => ZeroPage,
        0b010 => Immediate,
        0b011 => Absolute,
        0b100 => IndirectYIndexed,
        0b101 => ZeroPageXIndexed,
        0b110 => AbsoluteYIndexed,
        0b111 => AbsoluteXIndexed,
        _ => panic!("Unrecognised op code: {:02x}", (aaa << 5) | (bbb << 2) | 0x01),
    };
    (mnemonic, address_mode)
}

#[inline(always)]
fn match_cc_10(aaa: u8, bbb: u8) -> (Mnemonic, AddressMode) {
    let mnemonic = match aaa {
        0b000 => ASL,
        0b001 => ROL,
        0b010 => LSR,
        0b011 => ROR,
        0b100 => STX,
        0b101 => LDX,
        0b110 => DEC,
        0b111 => INC,
        _ => panic!("Unrecognised op code: {:02x}", (aaa << 5) | (bbb << 2) | 0b10),
    };
    let address_mode = match (bbb, mnemonic) {
        (0b000, _)   => Immediate,
        (0b001, _)   => ZeroPage,
        (0b010, _)   => Accumulator,
        (0b011, _)   => Absolute,
        (0b101, STX) => ZeroPageYIndexed,
        (0b101, LDX) => ZeroPageYIndexed,
        (0b101, _)   => ZeroPageXIndexed,
        (0b111, LDX) => AbsoluteYIndexed,
        (0b111, _)   => AbsoluteXIndexed,
        _ => panic!("Unrecognised op code: {:02x}", (aaa << 5) | (bbb << 2) | 0b10),
    };
    (mnemonic, address_mode)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Instruction {
    pub code: u8,
    pub mnemonic: Mnemonic,
    pub address_mode: AddressMode,
    pub cycles: isize,
}

impl Instruction {
    pub fn from_code(code: u8) -> Instruction {
        let (mnemonic, address_mode, cycles) = match code {
            0x00 => (BRK, Implied, 7),
            0x01 => (ORA, XIndexedIndirect, 6),
            0x05 => (ORA, ZeroPage, 3),
            0x06 => (ASL, ZeroPage, 5),
            0x08 => (PHP, Implied, 3),
            0x09 => (ORA, Immediate, 2),
            0x0a => (ASL, Accumulator, 2),
            0x0d => (ORA, Absolute, 4),
            0x0e => (ASL, Absolute, 6),
            0x10 => (BPL, Relative, 2),
            0x11 => (ORA, IndirectYIndexed, 5),
            0x15 => (ORA, ZeroPageXIndexed, 4),
            0x16 => (ASL, ZeroPageXIndexed, 6),
            0x18 => (CLC, Implied, 2),
            0x19 => (ORA, AbsoluteYIndexed, 4),
            0x1d => (ORA, AbsoluteXIndexed, 4),
            0x1e => (ASL, AbsoluteXIndexed, 7),
            0x20 => (JSR, Absolute, 6),
            0x21 => (AND, XIndexedIndirect, 6),
            0x24 => (BIT, ZeroPage, 3),
            0x25 => (AND, ZeroPage, 3),
            0x26 => (ROL, ZeroPage, 5),
            0x28 => (PLP, Implied, 4),
            0x29 => (AND, Immediate, 2),
            0x2a => (ROL, Accumulator, 2),
            0x2c => (BIT, Absolute, 4),
            0x2d => (AND, Absolute, 4),
            0x2e => (ROL, Absolute, 6),
            0x30 => (BMI, Relative, 2),
            0x31 => (AND, IndirectYIndexed, 5),
            0x35 => (AND, ZeroPageXIndexed, 4),
            0x36 => (ROL, ZeroPageXIndexed, 6),
            0x38 => (SEC, Implied, 2),
            0x39 => (AND, AbsoluteYIndexed, 4),
            0x3d => (AND, AbsoluteXIndexed, 4),
            0x3e => (ROL, AbsoluteXIndexed, 7),
            0x40 => (RTI, Implied, 6),
            0x41 => (EOR, XIndexedIndirect, 6),
            0x45 => (EOR, ZeroPage, 3),
            0x46 => (LSR, ZeroPage, 5),
            0x48 => (PHA, Implied, 3),
            0x49 => (EOR, Immediate, 2),
            0x4a => (LSR, Accumulator, 2),
            0x4c => (JMP, Absolute, 3),
            0x4d => (EOR, Absolute, 4),
            0x4e => (LSR, Absolute, 6),
            0x50 => (BVC, Relative, 2),
            0x51 => (EOR, IndirectYIndexed, 5),
            0x55 => (EOR, ZeroPageXIndexed, 4),
            0x56 => (LSR, ZeroPageXIndexed, 6),
            0x58 => (CLI, Implied, 2),
            0x59 => (EOR, AbsoluteYIndexed, 4),
            0x5d => (EOR, AbsoluteXIndexed, 4),
            0x5e => (LSR, AbsoluteXIndexed, 7),
            0x60 => (RTS, Implied, 6),
            0x61 => (ADC, XIndexedIndirect, 6),
            0x65 => (ADC, ZeroPage, 3),
            0x66 => (ROR, ZeroPage, 5),
            0x68 => (PLA, Implied, 4),
            0x69 => (ADC, Immediate, 2),
            0x6a => (ROR, Accumulator, 2),
            0x6c => (JMP, Indirect, 5),
            0x6d => (ADC, Absolute, 4),
            0x6e => (ROR, Absolute, 6),
            0x70 => (BVS, Relative, 2),
            0x71 => (ADC, IndirectYIndexed, 5),
            0x75 => (ADC, ZeroPageXIndexed, 4),
            0x76 => (ROR, ZeroPageXIndexed, 6),
            0x78 => (SEI, Implied, 2),
            0x79 => (ADC, AbsoluteYIndexed, 4),
            0x7d => (ADC, AbsoluteXIndexed, 4),
            0x7e => (ROR, AbsoluteXIndexed, 7),
            0x81 => (STA, XIndexedIndirect, 6),
            0x84 => (STY, ZeroPage, 3),
            0x85 => (STA, ZeroPage, 3),
            0x86 => (STX, ZeroPage, 3),
            0x88 => (DEY, Implied, 2),
            0x8a => (TXA, Implied, 2),
            0x8c => (STY, Absolute, 4),
            0x8d => (STA, Absolute, 4),
            0x8e => (STX, Absolute, 4),
            0x90 => (BCC, Relative, 2),
            0x91 => (STA, IndirectYIndexed, 6),
            0x94 => (STY, ZeroPageXIndexed, 4),
            0x95 => (STA, ZeroPageXIndexed, 4),
            0x96 => (STX, ZeroPageYIndexed, 4),
            0x98 => (TYA, Implied, 2),
            0x99 => (STA, AbsoluteYIndexed, 5),
            0x9a => (TXS, Implied, 2),
            0x9d => (STA, AbsoluteXIndexed, 5),
            0xa0 => (LDY, Immediate, 2),
            0xa1 => (LDA, XIndexedIndirect, 6),
            0xa2 => (LDX, Immediate, 2),
            0xa4 => (LDY, ZeroPage, 3),
            0xa5 => (LDA, ZeroPage, 3),
            0xa6 => (LDX, ZeroPage, 3),
            0xa8 => (TAY, Implied, 2),
            0xa9 => (LDA, Immediate, 2),
            0xaa => (TAX, Implied, 2),
            0xac => (LDY, Absolute, 4),
            0xad => (LDA, Absolute, 4),
            0xae => (LDX, Absolute, 4),
            0xb0 => (BCS, Relative, 2),
            0xb1 => (LDA, IndirectYIndexed, 5),
            0xb4 => (LDY, ZeroPageXIndexed, 4),
            0xb5 => (LDA, ZeroPageXIndexed, 4),
            0xb6 => (LDX, ZeroPageYIndexed, 4),
            0xb8 => (CLV, Implied, 2),
            0xb9 => (LDA, AbsoluteYIndexed, 4),
            0xba => (TSX, Implied, 2),
            0xbc => (LDY, AbsoluteXIndexed, 4),
            0xbd => (LDA, AbsoluteXIndexed, 4),
            0xbe => (LDX, AbsoluteYIndexed, 4),
            0xc0 => (CPY, Immediate, 2),
            0xc1 => (CMP, XIndexedIndirect, 6),
            0xc4 => (CPY, ZeroPage, 3),
            0xc5 => (CMP, ZeroPage, 3),
            0xc6 => (DEC, ZeroPage, 5),
            0xc8 => (INY, Implied, 2),
            0xc9 => (CMP, Immediate, 2),
            0xca => (DEX, Implied, 2),
            0xcc => (CPY, Absolute, 4),
            0xcd => (CMP, Absolute, 4),
            0xce => (DEC, Absolute, 3),
            0xd0 => (BNE, Relative, 2),
            0xd1 => (CMP, IndirectYIndexed, 5),
            0xd5 => (CMP, ZeroPageXIndexed, 4),
            0xd6 => (DEC, ZeroPageXIndexed, 6),
            0xd8 => (CLD, Implied, 2),
            0xd9 => (CMP, AbsoluteYIndexed, 4),
            0xdd => (CMP, AbsoluteXIndexed, 4),
            0xde => (DEC, AbsoluteXIndexed, 7),
            0xe0 => (CPX, Immediate, 2),
            0xe1 => (SBC, XIndexedIndirect, 6),
            0xe4 => (CPX, ZeroPage, 3),
            0xe5 => (SBC, ZeroPage, 3),
            0xe6 => (INC, ZeroPage, 5),
            0xe8 => (INX, Implied, 2),
            0xe9 => (SBC, Immediate, 2),
            0xea => (NOP, Implied, 2),
            0xec => (CPX, Absolute, 4),
            0xed => (SBC, Absolute, 4),
            0xee => (INC, Absolute, 6),
            0xf0 => (BEQ, Relative, 2),
            0xf1 => (SBC, IndirectYIndexed, 5),
            0xf5 => (SBC, ZeroPageXIndexed, 4),
            0xf6 => (INC, ZeroPageXIndexed, 6),
            0xf8 => (SED, Implied, 2),
            0xf9 => (SBC, AbsoluteYIndexed, 4),
            0xfd => (SBC, AbsoluteXIndexed, 4),
            0xfe => (INC, AbsoluteXIndexed, 7),
            _ => panic!("Unrecognised op code: {:02x}", code),
        };
//        let (mnemonic, address_mode) = match code {
//            0x00 => (BRK, Implied),
//            0x20 => (JSR, Absolute),
//            0x40 => (RTI, Implied),
//            0x60 => (RTS, Implied),
//
//            0x08 => (PHP, Implied),
//            0x28 => (PLP, Implied),
//            0x48 => (PHA, Implied),
//            0x68 => (PLA, Implied),
//            0x88 => (DEY, Implied),
//            0xa8 => (TAY, Implied),
//            0xc8 => (INY, Implied),
//            0xe8 => (INX, Implied),
//
//            0x18 => (CLC, Implied),
//            0x38 => (SEC, Implied),
//            0x58 => (CLI, Implied),
//            0x78 => (SEI, Implied),
//            0x98 => (TYA, Implied),
//            0xb8 => (CLV, Implied),
//            0xd8 => (CLD, Implied),
//            0xf8 => (SED, Implied),
//
//            0x8a => (TXA, Implied),
//            0x9a => (TXS, Implied),
//            0xaa => (TAX, Implied),
//            0xba => (TSX, Implied),
//            0xca => (DEX, Implied),
//            0xea => (NOP, Implied),
//
//            // Conditional Instructions
//            0x10 => (BPL, Relative),
//            0x30 => (BMI, Relative),
//            0x50 => (BVC, Relative),
//            0x70 => (BVS, Relative),
//            0x90 => (BCC, Relative),
//            0xb0 => (BCS, Relative),
//            0xd0 => (BNE, Relative),
//            0xf0 => (BEQ, Relative),
//
//            _ => {
//                let aaa = (code & 0b11100000) >> 5;
//                let bbb = (code & 0b00011100) >> 2;
//                let cc  = code & 0b00000011;
//                match cc {
//                    0b01 => match_cc_01(aaa, bbb),
//                    0b10 => match_cc_10(aaa, bbb),
//                    0b00 => match_cc_00(aaa, bbb),
//                    _ => panic!("Unrecognised op code: {:02x}", code),
//                }
//            },
//        };

        Instruction {
            code: code,
            mnemonic: mnemonic,
            address_mode: address_mode,
            cycles: cycles,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.mnemonic)
    }
}

use std::fmt;

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

#[inline(always)]
fn match_cc_00(aaa: u8, bbb: u8) -> (Mnemonic, AddressMode) {
    let mnemonic = match aaa {
        0b001 => Mnemonic::BIT,
        0b010 => Mnemonic::JMP,
        0b011 => Mnemonic::JMP,
        0b100 => Mnemonic::STY,
        0b101 => Mnemonic::LDY,
        0b110 => Mnemonic::CPY,
        0b111 => Mnemonic::CPX,
        _ => panic!("Unrecognised op code: {:#010x}", (aaa << 5) | (bbb << 2)),
    };
    let address_mode = match bbb {
        0b000 => AddressMode::Immediate,
        0b001 => AddressMode::ZeroPage,
        0b011 => AddressMode::Absolute,
        0b101 => AddressMode::ZeroPageXIndexed,
        0b111 => AddressMode::AbsoluteXIndexed,
        _ => panic!("Unrecognised op code: {:#010x}", (aaa << 5) | (bbb << 2)),
    };
    (mnemonic, address_mode)
}

#[inline(always)]
fn match_cc_01(aaa: u8, bbb: u8) -> (Mnemonic, AddressMode) {
    let mnemonic = match aaa {
        0b000 => Mnemonic::ORA,
        0b001 => Mnemonic::AND,
        0b010 => Mnemonic::EOR,
        0b011 => Mnemonic::ADC,
        0b100 => Mnemonic::STA,
        0b101 => Mnemonic::LDA,
        0b110 => Mnemonic::CMP,
        0b111 => Mnemonic::SBC,
        _ => panic!("Unrecognised op code: {:#010x}", (aaa << 5) | (bbb << 2) | 0x01),
    };
    let address_mode = match bbb {
        0b000 => AddressMode::XIndexedIndirect,
        0b001 => AddressMode::ZeroPage,
        0b010 => AddressMode::Immediate,
        0b011 => AddressMode::Absolute,
        0b100 => AddressMode::IndirectYIndexed,
        0b101 => AddressMode::ZeroPageXIndexed,
        0b110 => AddressMode::AbsoluteYIndexed,
        0b111 => AddressMode::AbsoluteXIndexed,
        _ => panic!("Unrecognised op code: {:#010x}", (aaa << 5) | (bbb << 2) | 0x01),
    };
    (mnemonic, address_mode)
}

#[inline(always)]
fn match_cc_10(aaa: u8, bbb: u8) -> (Mnemonic, AddressMode) {
    let mnemonic = match aaa {
        0b000 => Mnemonic::ASL,
        0b001 => Mnemonic::ROL,
        0b010 => Mnemonic::LSR,
        0b011 => Mnemonic::ROR,
        0b100 => Mnemonic::STX,
        0b101 => Mnemonic::LDX,
        0b110 => Mnemonic::DEC,
        0b111 => Mnemonic::INC,
        _ => panic!("Unrecognised op code: {:#010x}", (aaa << 5) | (bbb << 2) | 0b10),
    };
    let address_mode = match (bbb, mnemonic) {
        (0b000, _)             => AddressMode::Immediate,
        (0b001, _)             => AddressMode::ZeroPage,
        (0b010, _)             => AddressMode::Accumulator,
        (0b011, _)             => AddressMode::Absolute,
        (0b101, Mnemonic::STX) => AddressMode::ZeroPageYIndexed,
        (0b101, Mnemonic::LDX) => AddressMode::ZeroPageYIndexed,
        (0b101, _)             => AddressMode::ZeroPageXIndexed,
        (0b111, Mnemonic::LDX) => AddressMode::AbsoluteYIndexed,
        (0b111, _)             => AddressMode::AbsoluteXIndexed,
        _ => panic!("Unrecognised op code: {:#010x}", (aaa << 5) | (bbb << 2) | 0b10),
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
        println!("{:02x}", code);
        // Parse op codes according to this: http://www.llx.com/~nparker/a2/opcodes.html
        let (mnemonic, address_mode) = match code {
            0x00 => (Mnemonic::BRK, AddressMode::Implied),
            0x20 => (Mnemonic::JSR, AddressMode::Absolute),
            0x40 => (Mnemonic::RTI, AddressMode::Implied),
            0x60 => (Mnemonic::RTS, AddressMode::Implied),

            0x08 => (Mnemonic::PHP, AddressMode::Implied),
            0x28 => (Mnemonic::PLP, AddressMode::Implied),
            0x48 => (Mnemonic::PHA, AddressMode::Implied),
            0x68 => (Mnemonic::PLA, AddressMode::Implied),
            0x88 => (Mnemonic::DEY, AddressMode::Implied),
            0xa8 => (Mnemonic::TAY, AddressMode::Implied),
            0xc8 => (Mnemonic::INY, AddressMode::Implied),
            0xe8 => (Mnemonic::INX, AddressMode::Implied),

            0x18 => (Mnemonic::CLC, AddressMode::Implied),
            0x38 => (Mnemonic::SEC, AddressMode::Implied),
            0x58 => (Mnemonic::CLI, AddressMode::Implied),
            0x78 => (Mnemonic::SEI, AddressMode::Implied),
            0x98 => (Mnemonic::TYA, AddressMode::Implied),
            0xb8 => (Mnemonic::CLV, AddressMode::Implied),
            0xd8 => (Mnemonic::CLD, AddressMode::Implied),
            0xf8 => (Mnemonic::SED, AddressMode::Implied),

            0x8a => (Mnemonic::TXA, AddressMode::Implied),
            0x9a => (Mnemonic::TXS, AddressMode::Implied),
            0xaa => (Mnemonic::TAX, AddressMode::Implied),
            0xba => (Mnemonic::TSX, AddressMode::Implied),
            0xca => (Mnemonic::DEX, AddressMode::Implied),
            0xea => (Mnemonic::NOP, AddressMode::Implied),

            // Conditional Instructions
            0x10 => (Mnemonic::BPL, AddressMode::Relative),
            0x30 => (Mnemonic::BMI, AddressMode::Relative),
            0x50 => (Mnemonic::BVC, AddressMode::Relative),
            0x70 => (Mnemonic::BVS, AddressMode::Relative),
            0x90 => (Mnemonic::BCC, AddressMode::Relative),
            0xb0 => (Mnemonic::BCS, AddressMode::Relative),
            0xd0 => (Mnemonic::BNE, AddressMode::Relative),
            0xf0 => (Mnemonic::BEQ, AddressMode::Relative),

            _ => {
                let aaa = (code & 0b11100000) >> 5;
                let bbb = (code & 0b00011100) >> 2;
                let cc  = code & 0b00000011;
                match cc {
                    0b01 => match_cc_01(aaa, bbb),
                    0b10 => match_cc_10(aaa, bbb),
                    0b00 => match_cc_00(aaa, bbb),
                    _ => unreachable!(),
                }
            },
        };

        Instruction {
            code: code,
            mnemonic: mnemonic,
            address_mode: address_mode,
            cycles: 0,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.mnemonic)
    }
}

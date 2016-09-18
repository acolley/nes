//! A 6502 CPU Emulator

use super::instruction::{AddressMode, Instruction, Mnemonic};
use super::super::memory::{Memory};

// pub enum PpuAddr {
//     Control,
//     Mask,

// }

// enum NesAddr {
//     Ram(usize),
//     Ppu(PpuNesAddr),
//     Apu(usize),
//     Controller1,
//     Controller2,
//     Rom(usize),
// }

// impl NesAddr {
//     pub fn new(addr: u16) -> NesAddr {
//         match addr as usize {
//             x @ 0x0000 ... 0x1fff => NesAddr::Ram(x % 0x0800), // RAM is mirrored from 0x0800 - 0x2000
//             x @ 0x2000 ... 0x3fff => NesAddr::Ppu(0x2000 + (x % 8)),
//             0x4014 => NesAddr::Ppu(0x4014),
//             0x4015 => NesAddr::Apu(0x4015),
//             0x4016 => NesAddr::Controller1,
//             0x4017 => NesAddr::Controller2,
//             _ => panic!("Unrecognised address: ")
//         }
//     }
// }

// pub trait Memory {
//     fn read(&self, addr: u16) -> u8;
//     fn write(&self, addr: u16, x: u8);
//     fn crosses_page_boundary(before: u16, after: u16) -> bool;
// }

/// A struct holding all of the Registers
/// belonging to the 6502.
#[derive(Debug, Eq, PartialEq)]
pub struct Registers {
    pub pc: u16, // Program Counter
    pub sp: u16, // Stack Pointer
    pub a: u8, // Accumulator
    pub x: u8, // General purpose register
    pub y: u8, // General purpose register
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            pc: 0,
            sp: 0xfd,
            a: 0,
            x: 0,
            y: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flags {
    pub n: bool,
    pub z: bool,
    pub c: bool,
    pub i: bool,
    pub d: bool,
    pub v: bool,
    pub b: bool,
}

impl Flags {
    pub fn from_byte(byte: u8) -> Flags {
        Flags {
            n: (byte & 0b10000000) != 0,
            v: (byte & 0b01000000) != 0,
            b: (byte & 0b00010000) != 0,
            d: (byte & 0b00001000) != 0,
            i: (byte & 0b00000100) != 0,
            z: (byte & 0b00000010) != 0,
            c: (byte & 0b00000001) != 0,
        }
    }

    pub fn from_value_nz(x: u8) -> Flags {
        Flags {
            n: (x & 0x80) != 0,
            z: x == 0,
            .. Default::default()
        }
    }

    pub fn from_value_zc(x: u16) -> Flags {
        Flags {
            z: x == 0,
            c: (x & 0xff00) != 0,
            .. Default::default()
        }
    }

    pub fn from_value_nzc(x: u16) -> Flags {
        Flags {
            n: (x & 0x80) != 0,
            .. Flags::from_value_zc(x)
        }
    }

    pub fn from_value_nzcv(x: u16) -> Flags {
        panic!("Implement setting v flag");
        Flags::from_value_nzc(x)
    }

    pub fn as_byte(&self) -> u8 {
        (self.n as u8) << 7 |
        (self.v as u8) << 6 |
        (self.b as u8) << 4 |
        (self.d as u8) << 3 |
        (self.i as u8) << 2 |
        (self.z as u8) << 1 |
        self.c as u8
    }
}

impl Default for Flags {
    fn default() -> Flags {
        Flags {
            n: false,
            z: false,
            c: false,
            i: false,
            d: false,
            v: false,
            b: false,
        }
    }
}

fn pages_differ(a: u16, b: u16) -> bool {
    a & 0xff00 != b & 0xff00
}

pub struct Cpu {
    pub reg: Registers,
    pub flags: Flags,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg: Registers::new(),
            flags: Default::default(),
        }
    }

    /// Reset the CPU: http://wiki.nesdev.com/w/index.php/CPU_power_up_state
    pub fn reset(&mut self, mem: &Memory) {
        self.reg.pc = mem.read_u16(0xfffc);
        self.reg.sp = 0xfd;
        self.flags = Flags::from_byte(0x24);
    }

    fn next(&mut self, mem: &Memory) -> u8 {
        let x = mem.read(self.reg.pc);
        self.reg.pc += 1;
        x
    }

    fn next_u16(&mut self, mem: &Memory) -> u16 {
        let lo = self.next(mem);
        let hi = self.next(mem);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn absolute(&mut self, mem: &Memory) -> u16 {
        self.next_u16(mem)
    }

    fn indirect(&mut self, mem: &Memory) -> u16 {
        let base = self.next_u16(mem);
        let lo = mem.read(base);
        let hi = mem.read(base + 1);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn x_indexed_indirect(&self, mem: &Memory, base: u8) -> u16 {
        let indirect = base.wrapping_add(self.reg.x);
        let lo = mem.read(indirect as u16);
        let hi = mem.read(indirect.wrapping_add(1) as u16);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn indirect_indexed(&self, mem: &Memory, base: u8) -> u16 {
        let lo = mem.read(base as u16);
        let hi = mem.read(base.wrapping_add(1) as u16);
        let addr = ((lo as u16) | ((hi as u16) << 8)).wrapping_add(self.reg.y as u16);
        // (addr, pages_differ(addr, addr - self.reg.y as u16))
        addr
    }

    fn zero_page_indexed_x(&self, mem: &Memory, base: u8) -> u16 {
        base.wrapping_add(self.reg.x) as u16
    }

    fn zero_page_indexed_y(&self, mem: &Memory, base: u8) -> u16 {
        base.wrapping_add(self.reg.y) as u16
    }

    fn indexed_absolute_x(&self, mem: &Memory, base: u16) -> u16 {
        let addr = base.wrapping_add(self.reg.x as u16);
        // (addr, pages_differ(base, addr))
        addr
    }

    fn indexed_absolute_y(&self, mem: &Memory, base: u16) -> u16 {
        let addr = base.wrapping_add(self.reg.y as u16);
        // (addr, pages_differ(base, addr))
        addr
    }

    fn relative(&self, offset: u8) -> u16 {
        let addr = ((self.reg.pc as i32) + offset as i32) as u16;
        // (addr, pages_differ(self.reg.pc, addr))
        addr
    }

    fn php(&mut self, mem: &mut Memory) {
        let sp = self.flags.as_byte();
        self.push(mem, sp);
    }

    fn cmp(&mut self, x: u8, y: u8) {
        let value = (x as u16) - (y as u16);
        self.flags = Flags::from_value_nzc(value);
    }

    fn push(&mut self, mem: &mut Memory, x: u8) {
        mem.write(self.reg.sp, x);
        self.reg.sp -= 1;
    }

    fn push_u16(&mut self, mem: &mut Memory, x: u16) {
        let lo = x as u8;
        let hi = (x >> 8) as u8;
        self.push(mem, hi);
        self.push(mem, lo);
    }

    fn pop(&mut self, mem: &mut Memory) -> u8 {
        let value = mem.read(self.reg.sp);
        self.reg.sp += 1;
        value
    }

    fn pop_u16(&mut self, mem: &mut Memory) -> u16 {
        let lo = self.pop(mem);
        let hi = self.pop(mem);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn get_address_and_value(&mut self, mem: &mut Memory, address_mode: AddressMode) -> (Option<u16>, u8) {
        match address_mode {
            AddressMode::Accumulator => (None, self.reg.a),
            AddressMode::Absolute => {
                let addr = self.absolute(mem);
                (Some(addr), mem.read(addr))
            },
            AddressMode::AbsoluteXIndexed => {
                let base = self.absolute(mem);
                let addr = self.indexed_absolute_x(mem, base);
                (Some(addr), mem.read(addr))
            },
            AddressMode::AbsoluteYIndexed => {
                let base = self.absolute(mem);
                let addr = self.indexed_absolute_y(mem, base);
                (Some(addr), mem.read(addr))
            },
            AddressMode::Immediate => {
                (None, self.next(mem))
            },
            AddressMode::Indirect => {
                let addr = self.indirect(mem);
                (Some(addr), mem.read(addr))
            },
            AddressMode::XIndexedIndirect => {
                let base = self.next(mem);
                let addr = self.x_indexed_indirect(mem, base);
                (Some(addr), mem.read(addr))
            },
            AddressMode::IndirectYIndexed => {
                let base = self.next(mem);
                let addr = self.indirect_indexed(mem, base);
                (Some(addr), mem.read(addr))
            },
            AddressMode::Relative => {
                let offset = self.next(mem);
                let addr = self.relative(offset);
                (Some(addr), mem.read(addr))
            },
            AddressMode::ZeroPage => {
                let addr = self.next(mem) as u16;
                (Some(addr), mem.read(addr))
            },
            AddressMode::ZeroPageXIndexed => {
                let base = self.next(mem);
                let addr = self.zero_page_indexed_x(mem, base);
                (Some(addr), mem.read(addr))
            },
            AddressMode::ZeroPageYIndexed => {
                let base = self.next(mem);
                let addr = self.zero_page_indexed_y(mem, base);
                (Some(addr), mem.read(addr))
            },
            _ => panic!("No address or value for AddressMode: `{:?}`", address_mode)
        }
    }

    fn get_address(&mut self, mem: &mut Memory, address_mode: AddressMode) -> u16 {
        let (addr, _) = self.get_address_and_value(mem, address_mode);
        addr.expect(format!("No address for mode: {:?}", address_mode).as_str())
    }

    fn get_address_value(&mut self, mem: &mut Memory, address_mode: AddressMode) -> u8 {
        let (_, value) = self.get_address_and_value(mem, address_mode);
        value
    }

    fn with_address_modify<F>(&mut self, mem: &mut Memory, address_mode: AddressMode, f: F)
        where F: Fn(u8) -> (u8, Flags) {
        // TODO: also accept a bit mask that determines what flags are to be set
        // based on the result of the given function.
        match address_mode {
            AddressMode::Accumulator => {
                let (value, flags) = f(self.reg.a);
                self.reg.a = value;
                self.flags = flags;
            },
            AddressMode::Immediate => {
                let value = self.next(mem);
                f(value);
            },
            _ => {
                let (addr, value) = self.get_address_and_value(mem, address_mode);
                let (value, flags) = f(value);
                mem.write(addr.unwrap(), value);
                self.flags = flags;
            },
        }
    }

    pub fn current_instruction(&self, mem: &Memory) -> Instruction {
        let code = mem.read(self.reg.pc);
        Instruction::from_code(code)
    }

    pub fn next_instruction(&mut self, mem: &mut Memory) -> Instruction {
        let code = self.next(mem);
        Instruction::from_code(code)
    }

    pub fn step(&mut self, mem: &mut Memory) -> isize {
        println!("{}", self.reg.pc);
        let instruction = self.next_instruction(mem);
        println!("{:#x} {:?}", instruction.code, instruction.mnemonic);
        match instruction.mnemonic {
            Mnemonic::ADC => {
                let a = self.reg.a;
                let c = self.flags.c;
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = a as u16 + value as u16 + c as u16;
                    (value as u8, Flags::from_value_nzcv(value))
                });
            },
            Mnemonic::AND => {
                let a = self.reg.a;
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = a & value;
                    (value, Flags::from_value_nz(value))
                });
            },
            Mnemonic::ASL => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16) << 1;
                    (value as u8, Flags::from_value_nzc(value))
                });
            },
            Mnemonic::BCC => {
                let addr = self.get_address(mem, instruction.address_mode);
                if !self.flags.c {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BCS => {
                let addr = self.get_address(mem, instruction.address_mode);
                if self.flags.c {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BEQ => {
                let addr = self.get_address(mem, instruction.address_mode);
                if self.flags.z {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BIT => {
                let value = self.get_address_value(mem, instruction.address_mode);
                self.flags.n = (value & 0b10000000) != 0;
                self.flags.v = (value & 0b01000000) != 0;
                self.flags.z = (self.reg.a & value) != 0;
            },
            Mnemonic::BMI => {
                let addr = self.get_address(mem, instruction.address_mode);
                if self.flags.n {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BNE => {
                let addr = self.get_address(mem, instruction.address_mode);
                if !self.flags.z {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BPL => {
                let addr = self.get_address(mem, instruction.address_mode);
                if !self.flags.n {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BRK => {
                // Disable interrupts
                self.flags.i = true;

                let pc = self.reg.pc;
                self.push_u16(mem, pc);
                self.php(mem);
                self.reg.pc = mem.read_u16(0xfffe);
            },
            Mnemonic::BVC => {
                let addr = self.get_address(mem, instruction.address_mode);
                if !self.flags.v {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::BVS => {
                let addr = self.get_address(mem, instruction.address_mode);
                if self.flags.v {
                    self.reg.pc = addr;
                }
            },
            Mnemonic::CLC => {
                self.flags.c = false;
            },
            Mnemonic::CLD => {
                self.flags.d = false;
            },
            Mnemonic::CLI => {
                self.flags.i = false;
            },
            Mnemonic::CLV => {
                self.flags.v = false;
            },
            Mnemonic::CMP => {
                let value = self.get_address_value(mem, instruction.address_mode);
                let a = self.reg.a;
                self.cmp(a, value);
            },
            Mnemonic::CPX => {
                let x = self.reg.x;
                let value = self.get_address_value(mem, instruction.address_mode);
                self.cmp(x, value);
            },
            Mnemonic::CPY => {
                let y = self.reg.y;
                let value = self.get_address_value(mem, instruction.address_mode);
                self.cmp(y, value);
            },
            Mnemonic::DEC => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = value.wrapping_sub(1);
                    (value, Flags::from_value_nz(value))
                });
            },
            Mnemonic::DEX => {
                self.reg.x = self.reg.x.wrapping_sub(1);
                self.flags = Flags::from_value_nz(self.reg.x);
            },
            Mnemonic::DEY => {
                self.reg.y = self.reg.y.wrapping_sub(1);
                self.flags = Flags::from_value_nz(self.reg.y);
            },
            Mnemonic::EOR => {
                let value = self.get_address_value(mem, instruction.address_mode);
                self.reg.a = value ^ self.reg.a;
                self.flags = Flags::from_value_nz(self.reg.a);
            },
            Mnemonic::INC => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = value.wrapping_add(1);
                    (value, Flags::from_value_nz(value))
                });
            },
            Mnemonic::INX => {
                self.reg.x = self.reg.x.wrapping_add(1);
                self.flags = Flags::from_value_nz(self.reg.x);
            },
            Mnemonic::INY => {
                self.reg.y = self.reg.y.wrapping_add(1);
                self.flags = Flags::from_value_nz(self.reg.y);
            },
            Mnemonic::JMP => {
                let addr = self.get_address(mem, instruction.address_mode);
                self.reg.pc = addr;
            },
            Mnemonic::JSR => {
                let addr = self.get_address(mem, instruction.address_mode);
                let pc = self.reg.pc;
                self.push_u16(mem, pc);
                self.reg.pc = addr;
            },
            Mnemonic::LDA => {
                let value = self.get_address_value(mem, instruction.address_mode);
                self.reg.a = value;
                self.flags = Flags::from_value_nz(self.reg.a);
            },
            Mnemonic::LDX => {
                let value = self.get_address_value(mem, instruction.address_mode);
                self.reg.x = value;
                self.flags = Flags::from_value_nz(self.reg.x);
            },
            Mnemonic::LDY => {
                let value = self.get_address_value(mem, instruction.address_mode);
                self.reg.y = value;
                self.flags = Flags::from_value_nz(self.reg.y);
            },
            Mnemonic::LSR => {
                let value = self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16) >> 1;
                    (value as u8, Flags::from_value_zc(value))
                });
            },
            Mnemonic::NOP => {},
            Mnemonic::ORA => {
                let value = self.get_address_value(mem, instruction.address_mode);
                self.reg.a = self.reg.a | value;
                self.flags = Flags::from_value_nz(self.reg.a);
            },
            Mnemonic::PHA => {
                let a = self.reg.a;
                self.push(mem, a);
            },
            Mnemonic::PHP => {
                self.php(mem);
            },
            Mnemonic::PLA => {
                self.reg.a = self.pop(mem);
            },
            Mnemonic::PLP => {
                self.flags = Flags::from_byte(self.pop(mem));
            },
            Mnemonic::ROL => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16).rotate_left(1);
                    ((value as u8).rotate_left(1), Flags::from_value_nzc(value))
                });
            },
            Mnemonic::ROR => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16).rotate_right(1);
                    ((value as u8).rotate_right(1), Flags::from_value_nzc(value))
                });
            },
            Mnemonic::RTI => {
                self.flags = Flags::from_byte(self.pop(mem));
                self.reg.pc = self.pop_u16(mem);
            },
            Mnemonic::RTS => {
                self.reg.pc = self.pop_u16(mem) + 1;
            },
            Mnemonic::SBC => {
                let a = self.reg.a;
                let c = self.flags.c as u8;
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let new = (a as u16)
                        .wrapping_sub((value as u16))
                        .wrapping_sub(c as u16);
                    (a.wrapping_sub(value).wrapping_sub(c), Flags::from_value_nzcv(new))
                });
            },
            Mnemonic::SEC => {
                self.flags.c = true;
            },
            Mnemonic::SED => {
                self.flags.d = true;
            },
            Mnemonic::SEI => {
                self.flags.i = true;
            },
            Mnemonic::STA => {
                let addr = self.get_address(mem, instruction.address_mode);
                mem.write(addr, self.reg.a);
            },
            Mnemonic::STX => {
                let addr = self.get_address(mem, instruction.address_mode);
                mem.write(addr, self.reg.x);
            },
            Mnemonic::STY => {
                let addr = self.get_address(mem, instruction.address_mode);
                mem.write(addr, self.reg.y);
            },
            Mnemonic::TAX => {
                self.reg.x = self.reg.a;
            },
            Mnemonic::TAY => {
                self.reg.y = self.reg.a;
            },
            Mnemonic::TSX => {
                self.reg.x = self.reg.sp as u8;
            },
            Mnemonic::TXA => {
                self.reg.a = self.reg.x;
            },
            Mnemonic::TXS => {
                self.reg.sp = self.reg.x as u16;
            },
            Mnemonic::TYA => {
                self.reg.a = self.reg.y;
            },
        }
        instruction.cycles
    }
}

// impl Memory for Vec<u8> {
//     fn read(&self, addr: u16) -> u8 {
//         self[addr as usize]
//     }
//     fn write(&mut self, addr: u16, x: u8) {
//         self.data[addr as usize] = x;
//     }
// }

// #[test]
// fn test_indexed_indirect_x() {
//     let mut mem = Vec::new();
//     mem.resize(0x2000, 0);
//     mem[0x02] = 0x37;
//     mem[0x03] = 0x13;

//     // Actual data located at 0x1337
//     mem[0x1337] = 0xfe

//     let mut cpu = Cpu::new();
//     cpu.reg.x = 0x01;
//     let addr = cpu.indexed_indirect_x(0x10);
//     assert_eq!(addr, 0x1f);
// }

// #[test]
// fn test_indirect_indexed_y() {
//     let mut cpu = Cpu::new();
//     cpu.reg.y = 0x0f;
//     let addr = cpu.indirect_indexed_y(0x10);

// }
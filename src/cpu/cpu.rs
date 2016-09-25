//! A 6502 CPU Emulator

use super::instruction::{AddressMode, Instruction, Mnemonic};
use super::super::interconnect::{Interconnect};

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
    pub fn reset(&mut self, interconnect: &mut Interconnect) {
        self.reg.pc = interconnect.cpu_read_u16(0xfffc);
        self.reg.sp = 0xfd;
        self.flags = Flags::from_byte(0x24);
    }

    /// Read the next byte of memory and advance
    /// the PC Register by the same amount.
    #[inline(always)]
    fn next(&mut self, mem: &mut Interconnect) -> u8 {
        let x = self.peek(mem);
        self.reg.pc += 1;
        x
    }

    /// Read the next two bytes of memory and advance
    /// the PC Register by the same amount.
    #[inline(always)]
    fn next_u16(&mut self, mem: &mut Interconnect) -> u16 {
        let lo = self.next(mem);
        let hi = self.next(mem);
        (lo as u16) | ((hi as u16) << 8)
    }

    /// Peek at the following byte of memory.
    #[inline(always)]
    pub fn peek(&self, mem: &mut Interconnect) -> u8 {
        mem.cpu_read(self.reg.pc)
    }

    /// Peek at the following two bytes of memory.
    #[inline(always)]
    pub fn peek_u16(&self, mem: &mut Interconnect) -> u16 {
        let lo = self.peek(mem);
        let hi = self.skip_peek(1, mem);
        (lo as u16) | ((hi as u16) << 8)
    }

    pub fn skip_peek(&self, skip: usize, mem: &mut Interconnect) -> u8 {
        mem.cpu_read(self.reg.pc + skip as u16)
    }

    pub fn skip_peek_u16(&self, skip: usize, mem: &mut Interconnect) -> u16 {
        let lo = self.skip_peek(skip, mem);
        let hi = self.skip_peek(skip + 1, mem);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn absolute(&mut self, mem: &mut Interconnect) -> u16 {
        self.next_u16(mem)
    }

    fn indirect(&mut self, mem: &mut Interconnect) -> u16 {
        let base = self.next_u16(mem);
        let lo = mem.cpu_read(base);
        let hi = mem.cpu_read(base + 1);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn x_indexed_indirect(&self, mem: &mut Interconnect, base: u8) -> u16 {
        let indirect = base.wrapping_add(self.reg.x);
        let lo = mem.cpu_read(indirect as u16);
        let hi = mem.cpu_read(indirect.wrapping_add(1) as u16);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn indirect_indexed(&self, mem: &mut Interconnect, base: u8) -> (u16, isize) {
        let lo = mem.cpu_read(base as u16);
        let hi = mem.cpu_read(base.wrapping_add(1) as u16);
        let addr = ((lo as u16) | ((hi as u16) << 8)).wrapping_add(self.reg.y as u16);
        let cycles = if pages_differ(addr, addr - self.reg.y as u16) { 1 } else { 0 };
        (addr, cycles)
    }

    fn zero_page_indexed_x(&self, mem: &Interconnect, base: u8) -> u16 {
        base.wrapping_add(self.reg.x) as u16
    }

    fn zero_page_indexed_y(&self, mem: &Interconnect, base: u8) -> u16 {
        base.wrapping_add(self.reg.y) as u16
    }

    fn indexed_absolute_x(&self, mem: &Interconnect, base: u16) -> (u16, isize) {
        let addr = base.wrapping_add(self.reg.x as u16);
        let cycles = if pages_differ(base, addr) { 1 } else { 0 };
        (addr, cycles)
    }

    fn indexed_absolute_y(&self, mem: &Interconnect, base: u16) -> (u16, isize) {
        let addr = base.wrapping_add(self.reg.y as u16);
        let cycles = if pages_differ(base, addr) { 1 } else { 0 };
        (addr, cycles)
    }

    /// Relative Addressing Mode
    /// The operand 'offset' is interpreted
    /// as a signed byte and added to the
    /// current PC to give the final address.
    fn relative(&self, offset: u8) -> (u16, isize) {
        let addr = ((self.reg.pc as i32) + (offset as i8) as i32) as u16;
        let cycles = if pages_differ(self.reg.pc, addr) { 2 } else { 1 };
        (addr, cycles)
    }

    fn php(&mut self, mem: &mut Interconnect) {
        let sp = self.flags.as_byte();
        self.push(mem, sp);
    }

    fn cmp(&mut self, x: u8, y: u8) {
        let value = (x as u16) - (y as u16);
        self.flags = Flags::from_value_nzc(value);
    }

    fn push(&mut self, mem: &mut Interconnect, x: u8) {
        mem.cpu_write(self.reg.sp, x);
        self.reg.sp -= 1;
    }

    fn push_u16(&mut self, mem: &mut Interconnect, x: u16) {
        let lo = x as u8;
        let hi = (x >> 8) as u8;
        self.push(mem, hi);
        self.push(mem, lo);
    }

    fn pop(&mut self, mem: &mut Interconnect) -> u8 {
        let value = mem.cpu_read(self.reg.sp);
        self.reg.sp += 1;
        value
    }

    fn pop_u16(&mut self, mem: &mut Interconnect) -> u16 {
        let lo = self.pop(mem);
        let hi = self.pop(mem);
        (lo as u16) | ((hi as u16) << 8)
    }

    fn get_address_and_value(&mut self, mem: &mut Interconnect, address_mode: AddressMode) -> (Option<u16>, u8, isize) {
        match address_mode {
            AddressMode::Accumulator => (None, self.reg.a, 0),
            AddressMode::Absolute => {
                let addr = self.absolute(mem);
                (Some(addr), mem.cpu_read(addr), 0)
            },
            AddressMode::AbsoluteXIndexed => {
                let base = self.absolute(mem);
                let (addr, cycles) = self.indexed_absolute_x(mem, base);
                (Some(addr), mem.cpu_read(addr), cycles)
            },
            AddressMode::AbsoluteYIndexed => {
                let base = self.absolute(mem);
                let (addr, cycles) = self.indexed_absolute_y(mem, base);
                (Some(addr), mem.cpu_read(addr), cycles)
            },
            AddressMode::Immediate => {
                (None, self.next(mem), 0)
            },
            AddressMode::Indirect => {
                let addr = self.indirect(mem);
                (Some(addr), mem.cpu_read(addr), 0)
            },
            AddressMode::XIndexedIndirect => {
                let base = self.next(mem);
                let addr = self.x_indexed_indirect(mem, base);
                (Some(addr), mem.cpu_read(addr), 0)
            },
            AddressMode::IndirectYIndexed => {
                let base = self.next(mem);
                let (addr, cycles) = self.indirect_indexed(mem, base);
                (Some(addr), mem.cpu_read(addr), cycles)
            },
            AddressMode::Relative => {
                let offset = self.next(mem);
                let (addr, cycles) = self.relative(offset);
                (Some(addr), mem.cpu_read(addr), cycles)
            },
            AddressMode::ZeroPage => {
                let addr = self.next(mem) as u16;
                (Some(addr), mem.cpu_read(addr), 0)
            },
            AddressMode::ZeroPageXIndexed => {
                let base = self.next(mem);
                let addr = self.zero_page_indexed_x(mem, base);
                (Some(addr), mem.cpu_read(addr), 0)
            },
            AddressMode::ZeroPageYIndexed => {
                let base = self.next(mem);
                let addr = self.zero_page_indexed_y(mem, base);
                (Some(addr), mem.cpu_read(addr), 0)
            },
            _ => panic!("No address or value for AddressMode: `{:?}`", address_mode)
        }
    }

    fn get_address(&mut self, mem: &mut Interconnect, address_mode: AddressMode) -> (u16, isize) {
        match address_mode {
            AddressMode::Absolute => {
                (self.absolute(mem), 0)
            },
            AddressMode::AbsoluteXIndexed => {
                let base = self.absolute(mem);
                self.indexed_absolute_x(mem, base)
            },
            AddressMode::AbsoluteYIndexed => {
                let base = self.absolute(mem);
                self.indexed_absolute_y(mem, base)
            },
            AddressMode::Indirect => {
                (self.indirect(mem), 0)
            },
            AddressMode::XIndexedIndirect => {
                let base = self.next(mem);
                (self.x_indexed_indirect(mem, base), 0)
            },
            AddressMode::IndirectYIndexed => {
                let base = self.next(mem);
                self.indirect_indexed(mem, base)
            },
            AddressMode::Relative => {
                let offset = self.next(mem);
                self.relative(offset)
            },
            AddressMode::ZeroPage => {
                (self.next(mem) as u16, 0)
            },
            AddressMode::ZeroPageXIndexed => {
                let base = self.next(mem);
                (self.zero_page_indexed_x(mem, base), 0)
            },
            AddressMode::ZeroPageYIndexed => {
                let base = self.next(mem);
                (self.zero_page_indexed_y(mem, base), 0)
            },
            _ => panic!("No address for mode: {:?}", address_mode),
        }
    }

    fn get_address_value(&mut self, mem: &mut Interconnect, address_mode: AddressMode) -> (u8, isize) {
        let (_, value, page_cycles) = self.get_address_and_value(mem, address_mode);
        (value, page_cycles)
    }

    fn with_address_modify<F>(&mut self, mem: &mut Interconnect, address_mode: AddressMode, f: F) -> isize
        where F: Fn(u8) -> (u8, Flags) {
        // TODO: also accept a bit mask that determines what flags are to be set
        // based on the result of the given function.
        match address_mode {
            AddressMode::Accumulator => {
                let (value, flags) = f(self.reg.a);
                self.reg.a = value;
                self.flags = flags;
                0
            },
            AddressMode::Immediate => {
                let value = self.next(mem);
                f(value);
                0
            },
            _ => {
                let (addr, value, page_cycles) = self.get_address_and_value(mem, address_mode);
                let (value, flags) = f(value);
                mem.cpu_write(addr.unwrap(), value);
                self.flags = flags;
                page_cycles
            },
        }
    }

    pub fn current_instruction(&self, mem: &mut Interconnect) -> Instruction {
        let code = mem.cpu_read(self.reg.pc);
        Instruction::from_code(code)
    }

    pub fn next_instruction(&mut self, mem: &mut Interconnect) -> Instruction {
        let code = self.next(mem);
        Instruction::from_code(code)
    }

    pub fn step(&mut self, mem: &mut Interconnect) -> isize {
//        println!("{}", self.reg.pc);
        let instruction = self.next_instruction(mem);
//        println!("{:#x} {:?}", instruction.code, instruction.mnemonic);
        let page_cycles = match instruction.mnemonic {
            Mnemonic::ADC => {
                let a = self.reg.a as u16;
                let c = self.flags.c as u16;
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = a + value as u16 + c;
                    (value as u8, Flags::from_value_nzcv(value))
                })
            },
            Mnemonic::AND => {
                let a = self.reg.a;
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = a & value;
                    (value, Flags::from_value_nz(value))
                })
            },
            Mnemonic::ASL => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16) << 1;
                    (value as u8, Flags::from_value_nzc(value))
                })
            },
            Mnemonic::BCC => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if !self.flags.c {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BCS => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if self.flags.c {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BEQ => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if self.flags.z {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BIT => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.flags.n = (value & 0b10000000) != 0;
                self.flags.v = (value & 0b01000000) != 0;
                self.flags.z = (self.reg.a & value) != 0;
                page_cycles
            },
            Mnemonic::BMI => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if self.flags.n {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BNE => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if !self.flags.z {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BPL => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if !self.flags.n {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BRK => {
                // Disable interrupts
                self.flags.i = true;

                let pc = self.reg.pc;
                self.push_u16(mem, pc);
                self.php(mem);
                self.reg.pc = mem.cpu_read_u16(0xfffe);
                0
            },
            Mnemonic::BVC => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if !self.flags.v {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::BVS => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                if self.flags.v {
                    self.reg.pc = addr;
                }
                page_cycles
            },
            Mnemonic::CLC => {
                self.flags.c = false;
                0
            },
            Mnemonic::CLD => {
                self.flags.d = false;
                0
            },
            Mnemonic::CLI => {
                self.flags.i = false;
                0
            },
            Mnemonic::CLV => {
                self.flags.v = false;
                0
            },
            Mnemonic::CMP => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                let a = self.reg.a;
                self.cmp(a, value);
                page_cycles
            },
            Mnemonic::CPX => {
                let x = self.reg.x;
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.cmp(x, value);
                page_cycles
            },
            Mnemonic::CPY => {
                let y = self.reg.y;
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.cmp(y, value);
                page_cycles
            },
            Mnemonic::DEC => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = value.wrapping_sub(1);
                    (value, Flags::from_value_nz(value))
                })
            },
            Mnemonic::DEX => {
                self.reg.x = self.reg.x.wrapping_sub(1);
                self.flags = Flags::from_value_nz(self.reg.x);
                0
            },
            Mnemonic::DEY => {
                self.reg.y = self.reg.y.wrapping_sub(1);
                self.flags = Flags::from_value_nz(self.reg.y);
                0
            },
            Mnemonic::EOR => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.reg.a = value ^ self.reg.a;
                self.flags = Flags::from_value_nz(self.reg.a);
                page_cycles
            },
            Mnemonic::INC => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = value.wrapping_add(1);
                    (value, Flags::from_value_nz(value))
                })
            },
            Mnemonic::INX => {
                self.reg.x = self.reg.x.wrapping_add(1);
                self.flags = Flags::from_value_nz(self.reg.x);
                0
            },
            Mnemonic::INY => {
                self.reg.y = self.reg.y.wrapping_add(1);
                self.flags = Flags::from_value_nz(self.reg.y);
                0
            },
            Mnemonic::JMP => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                self.reg.pc = addr;
                page_cycles
            },
            Mnemonic::JSR => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                let pc = self.reg.pc;
                self.push_u16(mem, pc);
                self.reg.pc = addr;
                page_cycles
            },
            Mnemonic::LDA => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.reg.a = value;
                self.flags = Flags::from_value_nz(self.reg.a);
                page_cycles
            },
            Mnemonic::LDX => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.reg.x = value;
                self.flags = Flags::from_value_nz(self.reg.x);
                page_cycles
            },
            Mnemonic::LDY => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.reg.y = value;
                self.flags = Flags::from_value_nz(self.reg.y);
                page_cycles
            },
            Mnemonic::LSR => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16) >> 1;
                    (value as u8, Flags::from_value_zc(value))
                })
            },
            Mnemonic::NOP => { 0 },
            Mnemonic::ORA => {
                let (value, page_cycles) = self.get_address_value(mem, instruction.address_mode);
                self.reg.a = self.reg.a | value;
                self.flags = Flags::from_value_nz(self.reg.a);
                page_cycles
            },
            Mnemonic::PHA => {
                let a = self.reg.a;
                self.push(mem, a);
                0
            },
            Mnemonic::PHP => {
                self.php(mem);
                0
            },
            Mnemonic::PLA => {
                self.reg.a = self.pop(mem);
                0
            },
            Mnemonic::PLP => {
                self.flags = Flags::from_byte(self.pop(mem));
                0
            },
            Mnemonic::ROL => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16).rotate_left(1);
                    ((value as u8).rotate_left(1), Flags::from_value_nzc(value))
                })
            },
            Mnemonic::ROR => {
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let value = (value as u16).rotate_right(1);
                    ((value as u8).rotate_right(1), Flags::from_value_nzc(value))
                })
            },
            Mnemonic::RTI => {
                self.flags = Flags::from_byte(self.pop(mem));
                self.reg.pc = self.pop_u16(mem);
                0
            },
            Mnemonic::RTS => {
                self.reg.pc = self.pop_u16(mem) + 1;
                0
            },
            Mnemonic::SBC => {
                let a = self.reg.a;
                let c = self.flags.c as u8;
                self.with_address_modify(mem, instruction.address_mode, |value| {
                    let new = (a as u16)
                        .wrapping_sub((value as u16))
                        .wrapping_sub(c as u16);
                    (a.wrapping_sub(value).wrapping_sub(c), Flags::from_value_nzcv(new))
                })
            },
            Mnemonic::SEC => {
                self.flags.c = true;
                0
            },
            Mnemonic::SED => {
                self.flags.d = true;
                0
            },
            Mnemonic::SEI => {
                self.flags.i = true;
                0
            },
            Mnemonic::STA => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                mem.cpu_write(addr, self.reg.a);
                page_cycles
            },
            Mnemonic::STX => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                mem.cpu_write(addr, self.reg.x);
                page_cycles
            },
            Mnemonic::STY => {
                let (addr, page_cycles) = self.get_address(mem, instruction.address_mode);
                mem.cpu_write(addr, self.reg.y);
                page_cycles
            },
            Mnemonic::TAX => {
                self.reg.x = self.reg.a;
                0
            },
            Mnemonic::TAY => {
                self.reg.y = self.reg.a;
                0
            },
            Mnemonic::TSX => {
                self.reg.x = self.reg.sp as u8;
                0
            },
            Mnemonic::TXA => {
                self.reg.a = self.reg.x;
                0
            },
            Mnemonic::TXS => {
                self.reg.sp = self.reg.x as u16;
                0
            },
            Mnemonic::TYA => {
                self.reg.a = self.reg.y;
                0
            },
        };
        instruction.cycles + page_cycles
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
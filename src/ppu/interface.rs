struct StatusFlags {
    vblank: bool,
    writes: bool,
}

impl Default for StatusFlags {
    fn default() -> Self {
        StatusFlags {
            vblank: false,
            writes: false,
        }
    }
}

struct Control {
    name_table_address: u16,
    addr_inc: u16,
    sprite_pattern_table: u16,
    background_pattern_table: u16,
    sprite_x: u16,
    sprite_y: u16,
    nmi: bool,
}

impl Control {
    fn apply(&mut self, x: u8) {
        self.name_table_address = match x & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => unreachable!(),
        };
        self.addr_inc = if (x & 0b100) == 0 { 1 } else { 32 };
        self.sprite_pattern_table = if (x & 0b1000) == 0 { 0x0000 } else { 0x1000 };
        self.background_pattern_table = if (x & 0b10000) == 0 { 0x0000 } else { 0x1000 };
        self.sprite_y = if (x & 0b100000) == 0 { 8 } else { 16 };
        self.nmi = x & 0b10000000 != 0;
    }
}

impl Default for Control {
    fn default() -> Self {
        Control {
            name_table_address: 0x2000,
            addr_inc: 1,
            sprite_pattern_table: 0x0000,
            background_pattern_table: 0x0000,
            sprite_x: 8,
            sprite_y: 8,
            nmi: false,
        }
    }
}

enum ColourMode {
    Colour,
    Monochrome,
}

// https://wiki.nesdev.com/w/index.php/PPU_registers#PPUMASK
struct Mask {
    red: bool,
    green: bool,
    blue: bool,
    sprites: bool,
    background: bool,
    left_sprites: bool,
    left_background: bool,
    colour_mode: ColourMode,
}

impl Mask {
    pub fn apply(&mut self, x: u8) {
        self.colour_mode = if x & 1 == 0 {
            ColourMode::Colour
        } else {
            ColourMode::Monochrome
        };
        self.left_background = x & 0b10 != 0;
        self.left_sprites = x & 0b100 != 0;
        self.background = x & 0b1000 != 0;
        self.sprites = x & 0b10000 != 0;
        self.red = x & 0b100000 != 0;
        self.green = x & 0b1000000 != 0;
        self.blue = x & 0b10000000 != 0;
    }
}

impl Default for Mask {
    fn default() -> Self {
        Mask {
            red: false,
            green: false,
            blue: false,
            sprites: false,
            background: false,
            left_sprites: false,
            left_background: false,
            colour_mode: ColourMode::Colour,
        }
    }
}

/// This struct allows indirect communication
/// with the PPU from other components in the NES.
/// Allowing components to communicate without
/// having a direct reference to the PPU struct.
pub struct PpuInterface {
    mem: Vec<u8>,
    // Sprite Attribute RAM
    spr_ram: Vec<u8>,
    // The address in spr_ram that will be written
    // to when writing to 0x2004.
    spr_addr: u8,
    control: Control,
    mask: Mask,
    flags: StatusFlags,
    addr: u16,
}

impl PpuInterface {
    pub fn new() -> Self {
        PpuInterface {
            mem: vec![0; 0x4000],
            spr_ram: vec![0; 256],
            spr_addr: 0x00,
            control: Default::default(),
            mask: Mask::default(),
            flags: Default::default(),
            addr: 0,
        }
    }

    #[inline(always)]
    fn read_status_register(&mut self) -> u8 {
        // TODO: incorporate more flags here
        let sr =
            (self.flags.vblank as u8) << 7 |
            (self.flags.writes as u8) << 4;
        self.flags.vblank = false;
        sr
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        let addr = 0x2000 + (addr % 8);

        match addr {
            0x2000 | 0x2001 => panic!("Trying to read from write-only PPU register: {:04x}", addr),
            0x2002 => self.read_status_register(),
            0x2003 ... 0x2006 => panic!("Trying to read from write-only PPU register: {:04x}", addr),
            0x2007 => {
                // TODO: first read is invalid, only second
                // read returns the requested data as it is
                // buffered.
                let x = self.read(self.addr);
                self.addr += self.control.addr_inc;
                x
            },
            _ => panic!("Invalid PPU register read address: {:04x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = (addr % 0x4000) as usize;
        self.mem[addr]
    }

    pub fn write_register(&mut self, addr: u16, x: u8) {
        let addr = 0x2000 + (addr % 8);

        match addr {
            0x2000 => self.control.apply(x),
            0x2001 => self.mask.apply(x),
            0x2002 => panic!("Trying to write to read-only PPU status register: {:04x}", addr),
            0x2003 => self.spr_addr = x,
            0x2004 => self.spr_ram[self.spr_addr as usize] = x,
            0x2006 => {
                // Write the lower nybble of the PPU address
                // to be read from or written to with 0x2007.
                self.addr = (self.addr << 8) | (x as u16);
            },
            0x2007 => {
                let ppu_addr = self.addr;
                self.write(ppu_addr, x)
            },
            _ => panic!("Invalid PPU register write address: {:#x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, x: u8) {
        let addr = (addr % 0x4000) as usize;
        self.mem[addr] = x;
    }

    #[inline(always)]
    pub fn write_spr(&mut self, addr: u8, x: u8) {
        self.spr_ram[addr as usize] = x;
    }

    #[inline(always)]
    pub fn control(&self) -> &Control { &self.control }

    #[inline(always)]
    pub fn mask(&self) -> &Mask { &self.mask }
}

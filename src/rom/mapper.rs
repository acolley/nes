pub trait Mapper {
    fn map_sram(&self, addr: u16) -> usize;
    fn map_prg(&self, addr: u16) -> usize;
    fn map_chr(&self, addr: u16) -> usize;
}

/// NROM Cartridge Mapper
/// https://wiki.nesdev.com/w/index.php/NROM
pub struct Mapper0 {
    pub nprg: usize,
}

impl Mapper for Mapper0 {
    fn map_sram(&self, addr: u16) -> usize {
        (addr - 0x6000) as usize
    }

    fn map_prg(&self, addr: u16) -> usize {
        // NROM mapper can have one or two PRG ROM banks
        // given by self.nprg. If there is only one then
        // the first bank is mirrored.
        let offset = (addr - 0x8000) % (0x4000 * self.nprg as u16);
        offset as usize
    }

    fn map_chr(&self, addr: u16) -> usize {
        unimplemented!()
    }
}

/// MMC1 Mapper
/// https://wiki.nesdev.com/w/index.php/MMC1
pub struct Mapper1;

impl Mapper for Mapper1 {
    fn map_sram(&self, addr: u16) -> usize {
        0
    }

    fn map_prg(&self, addr: u16) -> usize {
        0
    }

    fn map_chr(&self, addr: u16) -> usize {
        0
    }
}
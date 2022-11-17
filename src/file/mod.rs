use crate::{
    cartridge_header::CartridgeHeader,
    file::{file_allocation_table::FileAllocationTableEntry, file_name_table::FileNameTable},
};
use zerocopy::LayoutVerified;

pub mod file_allocation_table;
pub mod file_name_table;

pub struct Files<'lt> {
    pub fnt: FileNameTable,
    pub fat: LayoutVerified<&'lt [u8], [FileAllocationTableEntry]>,
    pub rom: &'lt [u8],
}

impl<'lt> Files<'lt> {
    pub fn read(header: &CartridgeHeader, rom: &'lt [u8]) -> Option<Self> {
        let fnt = header.read_fnt(rom)?;
        let fat = header.read_fat(rom)?;

        Some(Self { fnt, fat, rom })
    }
}

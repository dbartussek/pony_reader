use crate::{byte_types::int::U32, cartridge_header::CartridgeHeader};
use byteorder::LittleEndian;
use serde::{Deserialize, Serialize};
use zerocopy::{AsBytes, FromBytes, LayoutVerified, Unaligned};


#[repr(C)]
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Debug, Serialize, Deserialize)]
pub struct FileAllocationTableEntry {
    pub start: U32<LittleEndian>,
    pub end: U32<LittleEndian>,
}

impl FileAllocationTableEntry {
    pub fn read_fat<'lt>(
        header: &CartridgeHeader,
        rom: &'lt [u8],
    ) -> Option<LayoutVerified<&'lt [u8], [FileAllocationTableEntry]>> {
        let base = header.fat.offset.get() as usize;
        let size = header.fat.size.get() as usize;
        let fat_raw = rom.get(base..(base + size))?;

        let fat = LayoutVerified::<_, [FileAllocationTableEntry]>::new_slice(fat_raw)?;
        Some(fat)
    }

    pub fn get_file(self, rom: &[u8]) -> Option<&[u8]> {
        rom.get((self.start.get() as usize)..(self.end.get() as usize))
    }
}

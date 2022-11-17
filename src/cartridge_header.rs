use crate::{
    byte_types::{
        embedded_string::EmbeddedString,
        int::{U16, U32},
    },
    file::{
        file_allocation_table::FileAllocationTableEntry, file_name_table::FileNameTable, Files,
    },
};
use byte_unit::{Byte, KIBIBYTE};
use byteorder::LittleEndian;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use zerocopy::{AsBytes, FromBytes, LayoutVerified, Unaligned};

#[repr(C)]
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct CartridgeHeader {
    pub title: EmbeddedString<12>,
    pub game_code: EmbeddedString<4>,
    pub maker_code: EmbeddedString<2>,
    pub unit_code: u8,
    pub encryption_seed_select: u8,
    pub device_capacity_raw: u8,

    #[derivative(Debug = "ignore")]
    #[serde(skip, default = "default_array")]
    pub _reserved_0: [u8; 7],
    #[derivative(Debug = "ignore")]
    #[serde(skip, default)]
    pub _reserved_1: u8,

    pub region: u8,
    pub rom_version: u8,

    pub autostart: u8,

    pub arm9: CartridgeHeaderCodeInfo,
    pub arm7: CartridgeHeaderCodeInfo,

    pub fnt: OffsetAndSize,
    pub fat: OffsetAndSize,

    pub arm9_overlay: OffsetAndSize,
    pub arm7_overlay: OffsetAndSize,

    pub port_40001a4_normal: U32<LittleEndian>,
    pub port_40001a4_key1: U32<LittleEndian>,

    pub icon_title_offset: U32<LittleEndian>,

    pub secure_area_checksum: U16<LittleEndian>,
    pub secure_area_delay: U16<LittleEndian>,

    pub arm9_auto_load: U32<LittleEndian>,
    pub arm7_auto_load: U32<LittleEndian>,

    pub secure_area_disable: EmbeddedString<8>,

    pub total_used_rom_size: U32<LittleEndian>,
    pub rom_header_size: U32<LittleEndian>,

    #[derivative(Debug = "ignore")]
    #[serde(skip, default = "default_array")]
    pub _unknown_1: [u8; 4],
    #[derivative(Debug = "ignore")]
    #[serde(skip, default = "default_array")]
    pub _reserved_3: [u8; 8],

    pub nand_end_of_rom: U16<LittleEndian>,
    pub nand_start_of_rw: U16<LittleEndian>,

    #[derivative(Debug = "ignore")]
    #[serde(skip, default = "default_array")]
    pub _reserved_4: [u8; 0x18],
    pub fast_boot: EmbeddedString<0x10>,

    #[derivative(Debug = "ignore")]
    #[serde(skip, default = "default_array")]
    pub nintendo_logo: [u8; 0x9C],
    #[derivative(Debug = "ignore")]
    #[serde(skip, default)]
    pub nintendo_logo_checksum: U16<LittleEndian>,

    pub header_checksum: U16<LittleEndian>,

    pub debug: OffsetAndSize,
    pub debug_ram_address: U32<LittleEndian>,
}

#[repr(C)]
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct CartridgeHeaderCodeInfo {
    pub rom_offset: U32<LittleEndian>,
    pub entry_address: U32<LittleEndian>,
    pub ram_address: U32<LittleEndian>,
    pub size: U32<LittleEndian>,
}

#[repr(C)]
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct OffsetAndSize {
    pub offset: U32<LittleEndian>,
    pub size: U32<LittleEndian>,
}

impl CartridgeHeader {
    pub fn device_capacity(&self) -> Byte {
        Byte::from_bytes((128 << self.device_capacity_raw) * KIBIBYTE)
    }

    pub fn read_fnt(&self, rom: &[u8]) -> Option<FileNameTable> {
        let base = self.fnt.offset.get() as usize;
        let size = self.fnt.size.get();
        if size == 0 {
            return Some(FileNameTable {
                main_table: vec![],
                sub_tables: vec![],
            });
        }

        FileNameTable::read(rom, base)
    }

    pub fn read_fat<'lt>(
        &self,
        rom: &'lt [u8],
    ) -> Option<LayoutVerified<&'lt [u8], [FileAllocationTableEntry]>> {
        FileAllocationTableEntry::read_fat(self, &rom)
    }

    pub fn read_files<'lt>(&self, rom: &'lt [u8]) -> Option<Files<'lt>> {
        Files::read(self, rom)
    }
}

fn default_array<T, const SIZE: usize>() -> [T; SIZE]
where
    T: Default + Copy,
{
    [T::default(); SIZE]
}

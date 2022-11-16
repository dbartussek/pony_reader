use crate::byte_types::{
    embedded_string::{DynamicEmbeddedString, EmbeddedStringMake},
    int::{U16, U32},
};
use byteorder::LittleEndian;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use zerocopy::{AsBytes, FromBytes, LayoutVerified, Unaligned};

#[repr(C)]
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct DirectoryMainTableEntry {
    pub offset_to_sub_table: U32<LittleEndian>,
    pub id_of_first_file: U16<LittleEndian>,

    /// If this is the root directory: total number of directories.
    /// If this is a subdirectory: ID of the parent directory.
    ///
    /// Directory IDs start at 0xF000
    pub total_or_parent: U16<LittleEndian>,
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct DirectoryMainTable<'lt>(pub &'lt [DirectoryMainTableEntry]);

impl<'lt> DirectoryMainTable<'lt> {
    pub fn wrap(slice: &'lt [u8]) -> Option<Self> {
        let (head, _) = LayoutVerified::<_, DirectoryMainTableEntry>::new_from_prefix(slice)?;
        let total = head.total_or_parent.get() as usize;
        let values =
            LayoutVerified::<_, [DirectoryMainTableEntry]>::new_slice_from_prefix(slice, total)
                .map(|(values, _)| values.into_slice())?;

        Some(Self(values))
    }
}

#[derive(Clone, Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub enum SubTableEntry {
    FileEntry {
        name: DynamicEmbeddedString<127>,
    },
    DirectoryEntry {
        name: DynamicEmbeddedString<127>,
        directory_id: U16<LittleEndian>,
    },
}

impl SubTableEntry {
    pub fn parse(slice: &[u8]) -> Option<(Self, &[u8])> {
        const DIRECTORY_BIT: u8 = 0x80;

        let marker = *slice.get(0)?;

        let directory = marker & DIRECTORY_BIT != 0;
        let length = (marker & !DIRECTORY_BIT) as usize;

        if length == 0 {
            return None;
        }

        let slice = slice.get(1..)?;
        let name = slice.get(..length)?;
        let name = DynamicEmbeddedString::from_slice(name)?;
        let slice = slice.get(length..)?;

        Some(if directory {
            let (directory_id, slice) =
                LayoutVerified::<_, U16<LittleEndian>>::new_from_prefix(slice)?;

            (
                Self::DirectoryEntry {
                    name,
                    directory_id: *directory_id,
                },
                slice,
            )
        } else {
            (Self::FileEntry { name }, slice)
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileNameTable {
    pub main_table: Vec<DirectoryMainTableEntry>,
    pub sub_tables: Vec<Vec<SubTableEntry>>,
}

impl FileNameTable {
    pub fn read(file: &[u8], start: usize) -> Option<Self> {
        let main_table = DirectoryMainTable::wrap(file.get(start..)?)?.0.to_vec();

        let mut sub_tables = Vec::with_capacity(main_table.len());

        for entry in &main_table {
            let mut sub_table_slice =
                file.get((start + (entry.offset_to_sub_table.get() as usize))..)?;
            let mut sub_table = Vec::new();

            loop {
                match SubTableEntry::parse(sub_table_slice) {
                    Some((sub_entry, tail)) => {
                        sub_table.push(sub_entry);
                        sub_table_slice = tail;
                    },
                    None => break,
                }
            }

            sub_table.shrink_to_fit();
            sub_tables.push(sub_table);
        }

        Some(Self {
            main_table,
            sub_tables,
        })
    }

    fn walk_directory<'lt, F>(
        &'lt self,
        function: &mut F,
        directory: usize,
        name_vec: &mut Vec<&'lt DynamicEmbeddedString<127>>,
    ) where
        F: FnMut(&[&DynamicEmbeddedString<127>], u16),
    {
        let meta = self.main_table.get(directory).unwrap();
        let directory = self.sub_tables.get(directory).unwrap();

        let mut file_id_counter = meta.id_of_first_file.get();

        for entry in directory {
            match entry {
                SubTableEntry::FileEntry { name } => {
                    name_vec.push(name);
                    function(name_vec.as_slice(), file_id_counter);
                    file_id_counter += 1;

                    name_vec.pop();
                },
                SubTableEntry::DirectoryEntry { name, directory_id } => {
                    name_vec.push(name);
                    function(name_vec.as_slice(), directory_id.get());
                    self.walk_directory(function, (directory_id.get() - 0xF000) as usize, name_vec);

                    name_vec.pop();
                },
            }
        }
    }

    /// Lists files and directories with their id
    pub fn walk<F>(&self, mut function: F)
    where
        F: FnMut(&[&DynamicEmbeddedString<127>], u16),
    {
        self.walk_directory(&mut function, 0, &mut vec![])
    }
}

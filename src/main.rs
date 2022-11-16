use crate::{
    byte_types::embedded_string::EmbeddedStringCommon, cartridge_header::CartridgeHeader,
    file_allocation_table::FileAllocationTableEntry, file_name_table::FileNameTable,
};
use itertools::Itertools;
use ron::ser::{PrettyConfig, PrettyNumberFormat};
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Deref,
    path::PathBuf,
};
use zerocopy::LayoutVerified;

pub mod byte_types;
pub mod cartridge_header;
pub mod file_allocation_table;
pub mod file_name_table;

fn pretty() -> PrettyConfig {
    let mut pretty = PrettyConfig::default();
    pretty.number_format = PrettyNumberFormat::Hex;
    pretty
}

fn main() {
    let _ = std::fs::create_dir_all("out");

    let rom = std::fs::read("pony/Pony Friends GER.nds").unwrap();
    let (header, _) =
        LayoutVerified::<_, CartridgeHeader>::new_from_prefix(rom.as_slice()).unwrap();
    std::fs::write(
        "out/header.ron",
        ron::ser::to_string_pretty(&*header, pretty()).unwrap(),
    )
    .unwrap();

    let file_name_table = FileNameTable::read(&rom, header.fnt.offset.get() as usize).unwrap();

    std::fs::write(
        "out/fnt.ron",
        ron::ser::to_string_pretty(&file_name_table, pretty()).unwrap(),
    )
    .unwrap();

    let file_allocation_table = FileAllocationTableEntry::read_fat(&header, &rom).unwrap();
    std::fs::write(
        "out/fat.ron",
        ron::ser::to_string_pretty(&*file_allocation_table, pretty()).unwrap(),
    )
    .unwrap();

    {
        let mut max_id = 0;

        let mut tree = BufWriter::new(File::create("out/files.txt").unwrap());
        file_name_table.walk(|path, id| {
            if id < 0xf000 {
                max_id = max_id.max(id);
                writeln!(tree, "/{}", path.iter().map(|s| s.as_str_lossy()).join("/")).unwrap();

                let file: &[u8] = file_allocation_table
                    .get(id as usize)
                    .unwrap()
                    .get_file(&rom)
                    .unwrap();
                let mut file_path = PathBuf::from("out/files");

                for e in path {
                    file_path.push(e.as_str_lossy().deref());
                }
                let _ = std::fs::create_dir_all(file_path.parent().unwrap());
                std::fs::write(file_path, file).unwrap();
            }
        });

        println!("Max file id: {}", max_id);
    }
}

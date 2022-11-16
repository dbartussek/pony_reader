use crate::{
    byte_types::embedded_string::EmbeddedStringCommon, cartridge_header::CartridgeHeader,
    file_name_table::FileNameTable,
};
use itertools::Itertools;
use ron::ser::{PrettyConfig, PrettyNumberFormat};
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use zerocopy::LayoutVerified;

pub mod byte_types;
pub mod cartridge_header;
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

    println!(
        "Cartridge size: {:}",
        header.device_capacity().get_appropriate_unit(true)
    );

    {
        let mut tree = BufWriter::new(File::create("out/files.txt").unwrap());
        file_name_table.walk(|path, id| {
            if id < 0xf000 {
                writeln!(tree, "/{}", path.iter().map(|s| s.as_str_lossy()).join("/")).unwrap();
            }
        });
    }
}

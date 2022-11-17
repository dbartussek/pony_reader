use itertools::Itertools;
use pony_reader::{
    byte_types::embedded_string::EmbeddedStringCommon, cartridge_header::CartridgeHeader,
};
use ron::ser::{PrettyConfig, PrettyNumberFormat};
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Deref,
    path::PathBuf,
};
use zerocopy::LayoutVerified;

fn pretty() -> PrettyConfig {
    let mut pretty = PrettyConfig::default();
    pretty.number_format = PrettyNumberFormat::Hex;
    pretty
}

fn read_rom() -> Vec<u8> {
    // let mut rom = std::fs::read("pony/Pony Friends GER.nds").unwrap();
    let mut rom = std::fs::read("pony/TinyFB.nds").unwrap();

    let min_len = std::mem::size_of::<CartridgeHeader>();
    if rom.len() < min_len {
        rom.resize(min_len, 0);
    }

    rom
}

fn main() {
    let _ = std::fs::create_dir_all("out");
    let rom = read_rom();

    let (header, _) =
        LayoutVerified::<_, CartridgeHeader>::new_from_prefix(rom.as_slice()).unwrap();
    std::fs::write(
        "out/header.ron",
        ron::ser::to_string_pretty(&*header, pretty()).unwrap(),
    )
    .unwrap();

    let arm9_base = header.arm9.rom_offset.get() as usize;
    let arm9_length = header.arm9.size.get() as usize;
    let arm9 = rom.get(arm9_base..(arm9_base + arm9_length)).unwrap();
    std::fs::write("out/arm9.bin", arm9).unwrap();

    let files = header.read_files(&rom).unwrap();

    std::fs::write(
        "out/fnt.ron",
        ron::ser::to_string_pretty(&files.fnt, pretty()).unwrap(),
    )
    .unwrap();

    std::fs::write(
        "out/fat.ron",
        ron::ser::to_string_pretty(&*files.fat, pretty()).unwrap(),
    )
    .unwrap();

    {
        let mut max_id = 0;

        let mut tree = BufWriter::new(File::create("out/files.txt").unwrap());
        files.fnt.walk(|path, id| {
            if id < 0xf000 {
                max_id = max_id.max(id);
                writeln!(tree, "/{}", path.iter().map(|s| s.as_str_lossy()).join("/")).unwrap();

                let file: &[u8] = files.fat.get(id as usize).unwrap().get_file(&rom).unwrap();
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

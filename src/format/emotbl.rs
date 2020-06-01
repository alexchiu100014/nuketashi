use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::Path;

use encoding_rs::SHIFT_JIS;

pub type Emotbl = HashMap<String, EmotblEntry>;

#[derive(Debug, Clone)]
pub struct EmotblEntry {
    pub path: String,
    pub attributes: Vec<i32>,
    pub primer: String,
}

fn read_cstr<R: Read>(mut reader: R, buffer: &mut [u8]) -> std::io::Result<&mut [u8]> {
    reader.read_exact(buffer)?;
    let mut buffer = &mut buffer[..];

    for (i, &c) in buffer.iter().enumerate() {
        if c == 0x00 {
            buffer = &mut buffer[0..i];
            break;
        }
    }

    Ok(buffer)
}

pub fn load_emotbl<P: AsRef<Path>>(path: P) -> std::io::Result<Emotbl> {
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    use crate::utils;

    let emotbl = File::open(path)?;
    let mut emotbl = BufReader::new(emotbl);

    let mut res = Emotbl::new();

    loop {
        let mut entry_name = [0u8; 32];
        let entry_name = read_cstr(&mut emotbl, &mut entry_name)?;
        let (entry_name, _, _) = SHIFT_JIS.decode(entry_name);

        let mut path = [0u8; 32];
        let path = read_cstr(&mut emotbl, &mut path)?;
        let (path, _, _) = SHIFT_JIS.decode(path);

        if entry_name.len() == 0 {
            break;
        }

        let attribute_offset = utils::io::read_i32(&mut emotbl)?;
        let primer_offset = utils::io::read_i32(&mut emotbl)?;

        let cur = emotbl.seek(SeekFrom::Current(0))?;
        
        emotbl.seek(SeekFrom::Start(attribute_offset as u64))?;

        let mut attributes = Vec::new();

        loop {
            let i = utils::io::read_i32(&mut emotbl)?;
            if i == -1 {
                break;
            }

            attributes.push(i);
        }

        emotbl.seek(SeekFrom::Start(primer_offset as u64))?;

        let mut primer = [0u8; 32];
        let primer = read_cstr(&mut emotbl, &mut primer)?;
        let (primer, _, _) = SHIFT_JIS.decode(primer);

        emotbl.seek(SeekFrom::Start(cur))?;

        res.insert(
            entry_name.into(),
            EmotblEntry {
                path: path.into(),
                attributes,
                primer: primer.into(),
            },
        );
    }

    Ok(res)
}

#[test]
fn test_emotbl() {
    let emotbl = load_emotbl("./blob/NUKITASHI_T.WAR/EMOTBL.BIN").unwrap();

    println!("{:#?}", emotbl);
}

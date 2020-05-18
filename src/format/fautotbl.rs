use std::io::{BufReader, Read};
use std::path::Path;

pub fn load_face_map<P: AsRef<Path>>(path: P) -> std::io::Result<(Vec<String>, Vec<String>)> {
    use encoding_rs::SHIFT_JIS;
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    use crate::utils;

    let fautotbl = File::open(path)?;
    let mut fautotbl = BufReader::new(fautotbl);

    let face_file_offset = utils::io::read_i32(&mut fautotbl)? as u64;
    let face_name_offset = utils::io::read_i32(&mut fautotbl)? as u64;

    let mut face_filenames = Vec::new();
    let mut face_names = Vec::new();

    fautotbl.seek(SeekFrom::Start(face_file_offset))?;

    while fautotbl.seek(SeekFrom::Current(0))? < face_name_offset {
        let mut filename: [u8; 32] = [0; 32];

        fautotbl.read_exact(&mut filename)?;
        let _face_id = utils::io::read_i32(&mut fautotbl)?;

        // remove zero-padding
        let filename = {
            let mut filename = &filename[..];
            for (i, &c) in filename.iter().enumerate() {
                if c == 0x00 {
                    filename = &filename[0..i];
                    break;
                }
            }
            filename
        };

        if filename.len() == 0 {
            break;
        }

        face_filenames.push(
            std::str::from_utf8(filename)
                .expect("should be UTF-8 or ASCII")
                .to_string(),
        );
    }

    fautotbl.seek(SeekFrom::Start(face_name_offset))?;

    loop {
        let mut name = vec![0u8; 40];

        fautotbl.read_exact(&mut name)?;
        let _face_id = utils::io::read_i32(&mut fautotbl)?;

        // remove zero-padding
        let name = {
            let mut name = &name[..];
            for (i, &c) in name.iter().enumerate() {
                if c == 0x00 {
                    name = &name[0..i];
                    break;
                }
            }
            name
        };

        if name.len() == 0 {
            break;
        }

        let (name, _, _) = SHIFT_JIS.decode(name);
        face_names.push(name.to_string());
    }

    Ok((face_filenames, face_names))
}

#[test]
fn test_fautotbl() {
    let (a, b) = load_face_map("./blob/NUKITASHI_T.WAR/FAUTOTBL.BIN").unwrap();

    assert_eq!(a.len(), b.len());

    assert_eq!(
        a,
        [
            "ASANE", "FUMINO", "HINAMI", "HITOURA", "IKUKO", "JUN", "KOUKI", "MISAKI", "NANASE",
            "RAN", "REI", "SAKIMORI", "TESHIMA", "TOUKA"
        ]
    );
    assert_eq!(
        b,
        [
            "麻沙音",
            "文乃",
            "ヒナミ",
            "仁浦",
            "郁子",
            "淳之介",
            "光姫",
            "美岬",
            "奈々瀬",
            "蘭",
            "礼",
            "防人老人",
            "手嶋",
            "桐香"
        ]
    );
}

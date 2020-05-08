// based on GARbro/ImageS25.cs

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use failure::Fail;

const S25_MAGIC: &[u8; 4] = b"S25\0";

pub struct S25Archive<A> {
    pub file: BufReader<A>,
    pub entries: Vec<Option<i32>>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "io error: {:?}", _0)]
    IoError(std::io::Error),
    #[fail(display = "invalid archive")]
    InvalidArchive,
    #[fail(display = "no entry")]
    NoEntry,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

fn read_i32<R: Read>(mut reader: R) -> std::io::Result<i32> {
    let mut buf = 0_i32.to_le_bytes();
    reader.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

impl S25Archive<File> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let file = File::open(path)?;
        let mut file = BufReader::new(file);

        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        if &magic_buf != S25_MAGIC {
            return Err(Error::InvalidArchive);
        }

        let total_entries = read_i32(&mut file)?;

        let mut entries = vec![];

        for _ in 0..total_entries {
            let mut offset = [0u8; 4];
            file.read_exact(&mut offset)?;
            let offset = i32::from_le_bytes(offset);

            entries.push(if offset == 0 { None } else { Some(offset) });
        }

        Ok(S25Archive { file, entries })
    }
}

impl<'a> S25Archive<std::io::Cursor<&'a [u8]>> {
    pub fn from_raw_bytes<'b>(bytes: &'b [u8]) -> Result<Self>
    where
        'b: 'a,
    {
        use std::io::Cursor;

        let mut file = BufReader::new(Cursor::new(bytes));

        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        if &magic_buf != S25_MAGIC {
            return Err(Error::InvalidArchive);
        }

        let total_entries = read_i32(&mut file)?;

        let mut entries = vec![];

        for _ in 0..total_entries {
            let mut offset = [0u8; 4];
            file.read_exact(&mut offset)?;
            let offset = i32::from_le_bytes(offset);

            entries.push(if offset == 0 { None } else { Some(offset) });
        }

        Ok(S25Archive { file, entries })
    }
}

impl<A> S25Archive<A> {
    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn total_layers(&self) -> usize {
        self.total_entries() / 100 + 1
    }
}

#[test]
fn unpack_s25() {
    let mut s25 = S25Archive::open("./blob/NUKITASHI_G1.WAR/TOUKA_01L.S25").unwrap();
    s25.load_image_metadata(0).unwrap();
    s25.load_image_metadata(101).unwrap();
    s25.load_image_metadata(201).unwrap();
}

#[derive(Clone, Copy, Debug)]
pub struct S25ImageMetadata {
    pub width: i32,
    pub height: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub incremental: bool,
    pub head: i32,
}

#[derive(Clone)]
pub struct S25Image {
    pub metadata: S25ImageMetadata,
    pub rgba_buffer: Vec<u8>,
}

impl<A> S25Archive<A>
where
    A: Read + Seek,
{
    pub fn load_image_metadata(&mut self, entry: usize) -> Result<S25ImageMetadata> {
        let offset = self
            .entries
            .get(entry)
            .copied()
            .flatten()
            .ok_or(Error::NoEntry)?;

        self.file.seek(SeekFrom::Start(offset as u64))?;

        let width = read_i32(&mut self.file)?;
        let height = read_i32(&mut self.file)?;
        let offset_x = read_i32(&mut self.file)?;
        let offset_y = read_i32(&mut self.file)?;
        let incremental = 0 != (read_i32(&mut self.file)? as u32 & 0x80000000);

        Ok(S25ImageMetadata {
            width,
            height,
            offset_x,
            offset_y,
            incremental,
            head: offset + 0x14,
        })
    }

    pub fn load_image(&mut self, entry: usize) -> Result<S25Image> {
        let metadata = self.load_image_metadata(entry)?;
        let mut buf = vec![0u8; (metadata.width * metadata.height * 4) as usize];

        self.unpack(&metadata, &mut buf)?;

        todo!()
    }

    fn unpack(&mut self, metadata: &S25ImageMetadata, buf: &mut [u8]) -> std::io::Result<()> {
        // データ開始位置にカーソルを移動
        self.file.seek(SeekFrom::Start(metadata.head as u64))?;

        if metadata.incremental {
            return self.unpack_incremental(metadata, buf);
        }

        Ok(())
    }

    fn unpack_incremental(
        &mut self,
        metadata: &S25ImageMetadata,
        buf: &mut [u8],
    ) -> std::io::Result<()> {
        unimplemented!()
    }
}

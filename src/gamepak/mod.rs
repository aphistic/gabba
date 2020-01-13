use std::fs;
use std::fmt;
use std::error::Error;
use std::fmt::Formatter;

pub mod header;

pub use header::Header;

pub struct GamePak {
    header: Header,
    data: Vec<u8>,
}

impl GamePak {
    pub fn load_from_file(path: &str) -> Result<GamePak, String> {
        match fs::read(path) {
            Ok(data) => GamePak::load(data),
            Err(e) => Err(String::from(e.description())),
        }
    }

    pub fn load(data: Vec<u8>) -> Result<GamePak, String> {
        let h = Header::load(&data)?;

        Ok(GamePak {
            header: h,
            data,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl fmt::Debug for GamePak {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GamePak{{ data: {}b }}", self.data.len())
    }
}


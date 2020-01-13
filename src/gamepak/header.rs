use std::fmt;

const MAGIC_NUMBER: u8 = 0x96;

const HEADER_SIZE: usize = 192;
const GAME_TITLE_SIZE: usize = 12;
const GAME_CODE_SIZE: usize = 4;
const MAKER_CODE_SIZE: usize = 2;

const H_GAME_TITLE: usize = 0x0A0;
const H_GAME_CODE: usize = 0x0AC;
const H_MAKER_CODE: usize = 0x0B0;
const H_MAGIC: usize = 0x0B2;

pub struct Header {
    game_title: String,
    game_code: String,
    maker_code: String,
}

impl Header {
    pub fn load(data: &[u8]) -> Result<Header, String> {
        if data.len() < HEADER_SIZE {
            return Err(format!("header is too small ({})", data.len()));
        }

        if data[H_MAGIC] != MAGIC_NUMBER {
            return Err(format!("magic number is invalid"));
        }

        Ok(
            Header {
                game_title: String::from_utf8_lossy(data[H_GAME_TITLE..H_GAME_TITLE + GAME_TITLE_SIZE].as_ref())
                    .to_string(),
                game_code: String::from_utf8_lossy(data[H_GAME_CODE..H_GAME_CODE + GAME_CODE_SIZE].as_ref())
                    .to_string(),
                maker_code: String::from_utf8_lossy(data[H_MAKER_CODE..H_MAKER_CODE + MAKER_CODE_SIZE].as_ref())
                    .to_string(),
            }
        )
    }

    pub fn game_title(&self) -> &str {
        &self.game_title
    }

    pub fn game_code(&self) -> &str {
        &self.game_code
    }

    pub fn maker_code(&self) -> &str {
        &self.maker_code
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "header")
    }
}
#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    TOMLParseError(toml::de::Error),
    TOMLStructureError,
}


impl From<toml::de::Error> for Error {
    fn from(other: toml::de::Error) -> Self {
        Self::TOMLParseError(other)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Self::IOError(other)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(err) => {
                write!(f, "I/O error: {}", err)
            }
            Self::TOMLParseError(err) => {
                write!(f, "Error parsing TOML: {}", err)
            },
            Self::TOMLStructureError => {
                write!(f, "Strings TOML must be flat string/string table")
            },
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Encoding(os_str_bytes::EncodingError),
    Processing(String),
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::Io(e) => Some(e),
            Error::Encoding(e) => Some(e),
            Error::Processing(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Encoding(e) => e.fmt(f),
            Error::Processing(_) => self.fmt(f),
        }
    }
}

impl From<os_str_bytes::EncodingError> for Error {
    fn from(v: os_str_bytes::EncodingError) -> Self {
        Self::Encoding(v)
    }
}

impl From<std::io::Error> for Error {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

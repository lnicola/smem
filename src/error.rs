#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Processing(String),
    ParseSize,
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::Io(e) => Some(e),
            Error::Processing(_) => None,
            Error::ParseSize => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Processing(_) => self.fmt(f),
            Error::ParseSize => self.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

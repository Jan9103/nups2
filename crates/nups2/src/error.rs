use std::fmt::Display;

pub enum Nups2Error {
    IoError(std::io::Error),
    Other(&'static str),
}

impl Display for Nups2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nups2Error::IoError(error) => error.fmt(f),
            Nups2Error::Other(m) => write!(f, "Error: {}", m),
        }
    }
}

impl From<std::io::Error> for Nups2Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<&'static str> for Nups2Error {
    fn from(value: &'static str) -> Self {
        Self::Other(value)
    }
}

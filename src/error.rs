use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Failed to connect to the display (e.g. missing DISPLAY, XWayland down).
    Connection(String),
    /// Server query failed.
    Query(String),
    /// Platform without implementation.
    Unsupported(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(msg) => write!(f, "display connection failed: {msg}"),
            Self::Query(msg) => write!(f, "position query failed: {msg}"),
            Self::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

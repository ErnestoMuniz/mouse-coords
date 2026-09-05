use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Falha ao conectar no display (ex: DISPLAY ausente, XWayland fora do ar).
    Connection(String),
    /// Query ao servidor falhou.
    Query(String),
    /// Plataforma sem implementação.
    Unsupported(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(msg) => write!(f, "conexão com display falhou: {msg}"),
            Self::Query(msg) => write!(f, "query da posição falhou: {msg}"),
            Self::Unsupported(msg) => write!(f, "não suportado: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

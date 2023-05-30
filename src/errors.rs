use std::borrow::Cow;
pub use std::error::Error as StdError;
use std::fmt::{self, Formatter};
use std::sync::Arc;

type ErrorMsg = Cow<'static, str>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: Option<ErrorMsg>,
    pub source: Option<Arc<dyn StdError + Send + Sync + 'static>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Configuration,
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ErrorKind::Configuration => f.write_str("Configuration error"),
            ErrorKind::Other => f.write_str("Error"),
        }
    }
}

impl Error {
    pub fn new<T: Into<ErrorMsg>>(message: T) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: Some(message.into()),
            source: None,
        }
    }

    pub fn with_source(self, source: Arc<dyn StdError + Send + Sync + 'static>) -> Self {
        Self {
            source: Some(source),
            ..self
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or("Error"))?;
        if let Some(source) = &self.source {
            write!(f, ". (Source {source})")?;
        }

        Ok(())
    }
}

impl StdError for Error {}

pub trait ResultExt<T> {
    fn error<M: Into<ErrorMsg>>(self, message: M) -> Result<T>;
    fn or_error<M: Into<ErrorMsg>, F: FnOnce() -> M>(self, f: F) -> Result<T>;
}

impl<T, E: StdError + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
    fn error<M: Into<ErrorMsg>>(self, message: M) -> Result<T> {
        self.map_err(|e| Error {
            kind: ErrorKind::Other,
            message: Some(message.into()),
            source: Some(Arc::new(e)),
        })
    }

    fn or_error<M: Into<ErrorMsg>, F: FnOnce() -> M>(self, f: F) -> Result<T> {
        self.map_err(|e| Error {
            kind: ErrorKind::Other,
            message: Some(f().into()),
            source: Some(Arc::new(e)),
        })
    }
}

pub trait OptionExt<T> {
    fn error<M: Into<ErrorMsg>>(self, message: M) -> Result<T>;
    fn or_error<M: Into<ErrorMsg>, F: FnOnce() -> M>(self, f: F) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn error<M: Into<ErrorMsg>>(self, message: M) -> Result<T> {
        self.ok_or_else(|| Error {
            kind: ErrorKind::Other,
            message: Some(message.into()),
            source: None,
        })
    }

    fn or_error<M: Into<ErrorMsg>, F: FnOnce() -> M>(self, f: F) -> Result<T> {
        self.ok_or_else(|| Error {
            kind: ErrorKind::Other,
            message: Some(f().into()),
            source: None,
        })
    }
}

pub trait ToSerdeError<T> {
    fn serde_error<E: serde::de::Error>(self) -> Result<T, E>;
}

impl<T, F> ToSerdeError<T> for Result<T, F>
where
    F: fmt::Display,
{
    fn serde_error<E: serde::de::Error>(self) -> Result<T, E> {
        self.map_err(E::custom)
    }
}

pub struct BoxedError(pub Box<dyn StdError + Send + Sync + 'static>);

impl fmt::Debug for BoxedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl StdError for BoxedError {}

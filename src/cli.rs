use std::env::var_os;
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

pub(crate) struct EnvFailure {
    pub(crate) var: &'static str,
    pub(crate) err: EnvError,
}

pub(crate) enum EnvError {
    Missing,
    Empty,
    BadUnicode(Utf8Error),
}

pub(crate) fn require_noempty_utf8_env(var: &'static str) -> Result<String, EnvFailure> {
    match var_os(var) {
        None => Err(EnvError::Missing),
        Some(oss) => {
            if oss.is_empty() {
                Err(EnvError::Empty)
            } else {
                String::from_utf8(oss.into_encoded_bytes())
                    .map_err(|err| EnvError::BadUnicode(err.utf8_error()))
            }
        }
    }
    .map_err(|err| EnvFailure { var, err })
}

impl Display for EnvFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Environment variable {} ", self.var)?;

        match self.err {
            EnvError::Missing => write!(f, "missing"),
            EnvError::Empty => write!(f, "is empty"),
            EnvError::BadUnicode(err) => write!(f, "is not valid UTF-8: {}", err),
        }
    }
}

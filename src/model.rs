use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Level {
    Debug,
    Info,
    Warning,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Debug => write!(f, "Debug"),
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl TryFrom<i8> for Level {
    type Error = ();

    fn try_from(v: i8) -> Result<Self, ()> {
        match v {
            x if x == Self::Debug as i8 => Ok(Self::Debug),
            x if x == Self::Info as i8 => Ok(Self::Info),
            x if x == Self::Warning as i8 => Ok(Self::Warning),
            x if x == Self::Error as i8 => Ok(Self::Error),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogRecord {
    pub level: Level,
    pub time: u64,
    pub message: String,
}

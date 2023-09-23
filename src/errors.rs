use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TransformerError {
    UnknownTranformer(String, String),
    TooManyArguments(&'static str, usize, usize),
    TooFewArguments(&'static str, usize, usize),
    InvalidValueType(&'static str, &'static str),
    InvalidArgumentType(&'static str, String, &'static str),
}

impl Error for TransformerError {}

impl fmt::Display for TransformerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownTranformer(fun, val) => {
                write!(f, "{fun} transformer not found for value {val}")
            }
            Self::TooManyArguments(fun, r, g) => write!(f, "{fun} needs {r} arguments {g} given"),
            Self::TooFewArguments(fun, r, g) => write!(f, "{fun} needs {r} arguments {g} given"),
            Self::InvalidValueType(fun, t) => write!(f, "{fun} can only tranform {t} type values"),
            Self::InvalidArgumentType(fun, g, t) => {
                write!(f, "{fun} argument {g} needs to be of {t} type")
            }
        }
    }
}

use std::{error::Error, fmt};

/// Errors for the transformers
#[derive(Debug)]
pub enum TransformerError {
    /// The transformer with the name doesn't exist
    UnknownTranformer(String, String),
    /// Number of arguments is more than required
    TooManyArguments(&'static str, usize, usize),
    /// Not enough arguments for the transformer
    TooFewArguments(&'static str, usize, usize),
    /// The transformer cannot transform the given type
    InvalidValueType(&'static str, &'static str),
    /// The argument provided is not the correct type
    InvalidArgumentType(&'static str, String, &'static str),
}

impl Error for TransformerError {}

impl fmt::Display for TransformerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownTranformer(fun, val) => {
                write!(f, "{fun} transformer not found for value {val}")
            }
            Self::TooManyArguments(fun, r, g) => {
                write!(f, "{fun} takes at max {r} arguments {g} given")
            }
            Self::TooFewArguments(fun, r, g) => {
                write!(f, "{fun} needs at least {r} arguments {g} given")
            }
            Self::InvalidValueType(fun, t) => write!(f, "{fun} can only tranform {t} type values"),
            Self::InvalidArgumentType(fun, g, t) => {
                write!(f, "{fun} argument {g} needs to be of {t} type")
            }
        }
    }
}

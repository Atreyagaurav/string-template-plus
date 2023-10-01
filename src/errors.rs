use std::{error::Error, fmt};

/// Errors for the render template
#[derive(Debug)]
pub enum RenderTemplateError {
    /// The Template is not correctly formatted,
    InvalidFormat(String),
    /// Variable not found
    VariableNotFound(String),
    /// Any of the multiple Variables not found
    AllVariablesNotFound(Vec<String>),
    /// Error from Transformers
    TransformerError(TransformerError),
}

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

impl Error for RenderTemplateError {}
impl Error for TransformerError {}

impl From<TransformerError> for RenderTemplateError {
    fn from(item: TransformerError) -> Self {
        Self::TransformerError(item)
    }
}

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

impl fmt::Display for RenderTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidFormat(fstr) => {
                write!(f, "Invalid Template format: {fstr}")
            }
            Self::VariableNotFound(var) => {
                write!(f, "Variable {var} not found")
            }
            Self::AllVariablesNotFound(vars) => {
                write!(f, "None of the variables {vars:?} found")
            }
            Self::TransformerError(e) => e.fmt(f),
        }
    }
}

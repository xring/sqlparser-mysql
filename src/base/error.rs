use std::fmt;

use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use nom::InputLength;

/// [nom::branch::alt] return the last error of the branch by default
///
/// With a custom error type it is possible to have
/// [nom::branch::alt] return the error of the parser
/// that went the farthest in the input data.
///
/// There is little difference between [ParseSQLError] and [nom::error::VerboseError]
#[derive(Clone, Debug, PartialEq)]
pub struct ParseSQLError<I>
where
    I: InputLength,
{
    pub errors: Vec<(I, ParseSQLErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
/// Error context for `ParseSQLError`
pub enum ParseSQLErrorKind {
    /// Static string added by the `context` function
    Context(&'static str),
    /// Indicates which character was expected by the `char` function
    Char(char),
    /// Error kind given by various nom parsers
    Nom(ErrorKind),
}

impl<I> ParseError<I> for ParseSQLError<I>
where
    I: InputLength,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        ParseSQLError {
            errors: vec![(input, ParseSQLErrorKind::Nom(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, ParseSQLErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        ParseSQLError {
            errors: vec![(input, ParseSQLErrorKind::Char(c))],
        }
    }

    fn or(self, other: Self) -> Self {
        if self.errors[0].0.input_len() >= other.errors[0].0.input_len() {
            other
        } else {
            self
        }
    }
}

impl<I: nom::InputLength> ContextError<I> for ParseSQLError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ParseSQLErrorKind::Context(ctx)));
        other
    }
}

impl<I: InputLength, E> FromExternalError<I, E> for ParseSQLError<I> {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}

impl<I: fmt::Display + InputLength> fmt::Display for ParseSQLError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Parse error:")?;
        for (input, error) in &self.errors {
            match error {
                ParseSQLErrorKind::Nom(e) => writeln!(f, "{:?} at: {}", e, input)?,
                ParseSQLErrorKind::Char(c) => writeln!(f, "expected '{}' at: {}", c, input)?,
                ParseSQLErrorKind::Context(s) => writeln!(f, "in section '{}', at: {}", s, input)?,
            }
        }

        Ok(())
    }
}

impl<I: fmt::Debug + fmt::Display + InputLength> std::error::Error for ParseSQLError<I> {}

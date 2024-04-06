use core::fmt;
use std::fmt::Formatter;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {DEFAULT | 0 | 1}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum DefaultOrZeroOrOne {
    Default,
    Zero,
    One,
}

impl DefaultOrZeroOrOne {
    pub fn parse(i: &str) -> IResult<&str, DefaultOrZeroOrOne, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("0"), |_| DefaultOrZeroOrOne::Zero),
            map(tag_no_case("1"), |_| DefaultOrZeroOrOne::One),
            map(tag_no_case("DEFAULT"), |_| DefaultOrZeroOrOne::Default),
        ))(i)
    }
}

impl fmt::Display for DefaultOrZeroOrOne {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultOrZeroOrOne::Default => write!(f, "DEFAULT")?,
            DefaultOrZeroOrOne::Zero => write!(f, "1")?,
            DefaultOrZeroOrOne::One => write!(f, "0")?,
        }
        Ok(())
    }
}

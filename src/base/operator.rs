use std::fmt;
use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::error::ParseSQLError;

/// Parse binary comparison operators
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Operator {
    Not,
    And,
    Or,
    Like,
    NotLike,
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    In,
    NotIn,
    Is,
}

impl Operator {
    pub fn parse(i: &str) -> IResult<&str, Operator, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("NOT_LIKE"), |_| Operator::NotLike),
            map(tag_no_case("LIKE"), |_| Operator::Like),
            map(tag_no_case("!="), |_| Operator::NotEqual),
            map(tag_no_case("<>"), |_| Operator::NotEqual),
            map(tag_no_case(">="), |_| Operator::GreaterOrEqual),
            map(tag_no_case("<="), |_| Operator::LessOrEqual),
            map(tag_no_case("="), |_| Operator::Equal),
            map(tag_no_case("<"), |_| Operator::Less),
            map(tag_no_case(">"), |_| Operator::Greater),
            map(tag_no_case("IN"), |_| Operator::In),
        ))(i)
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op = match *self {
            Operator::Not => "NOT",
            Operator::And => "AND",
            Operator::Or => "OR",
            Operator::Like => "LIKE",
            Operator::NotLike => "NOT_LIKE",
            Operator::Equal => "=",
            Operator::NotEqual => "!=",
            Operator::Greater => ">",
            Operator::GreaterOrEqual => ">=",
            Operator::Less => "<",
            Operator::LessOrEqual => "<=",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::Is => "IS",
        };
        write!(f, "{}", op)
    }
}

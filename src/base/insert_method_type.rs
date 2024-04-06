use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// { NO | FIRST | LAST }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InsertMethodType {
    No,
    First,
    Last,
}

impl InsertMethodType {
    pub fn parse(i: &str) -> IResult<&str, InsertMethodType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("NO"), |_| InsertMethodType::No),
            map(tag_no_case("FIRST"), |_| InsertMethodType::First),
            map(tag_no_case("LAST"), |_| InsertMethodType::Last),
        ))(i)
    }
}

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {VISIBLE | INVISIBLE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum VisibleType {
    Visible,
    Invisible,
}

impl VisibleType {
    pub fn parse(i: &str) -> IResult<&str, VisibleType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("VISIBLE"), |_| VisibleType::Visible),
            map(tag_no_case("INVISIBLE"), |_| VisibleType::Invisible),
        ))(i)
    }
}

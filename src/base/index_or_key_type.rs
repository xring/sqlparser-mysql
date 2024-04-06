use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {INDEX | KEY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexOrKeyType {
    Index,
    Key,
}

impl IndexOrKeyType {
    /// {INDEX | KEY}
    pub fn parse(i: &str) -> IResult<&str, IndexOrKeyType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("KEY"), |_| IndexOrKeyType::Key),
            map(tag_no_case("INDEX"), |_| IndexOrKeyType::Index),
        ))(i)
    }
}

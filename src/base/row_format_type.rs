use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RowFormatType {
    Default,
    Dynamic,
    Fixed,
    Compressed,
    Redundant,
    Compact,
}

impl RowFormatType {
    pub fn parse(i: &str) -> IResult<&str, RowFormatType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DEFAULT"), |_| RowFormatType::Default),
            map(tag_no_case("DYNAMIC"), |_| RowFormatType::Dynamic),
            map(tag_no_case("FIXED"), |_| RowFormatType::Fixed),
            map(tag_no_case("COMPRESSED"), |_| RowFormatType::Compressed),
            map(tag_no_case("REDUNDANT"), |_| RowFormatType::Redundant),
            map(tag_no_case("COMPACT"), |_| RowFormatType::Compact),
        ))(i)
    }
}

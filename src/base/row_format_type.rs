use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// parse `ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RowFormatType {
    Default,
    Dynamic,
    Fixed,
    Compressed,
    Redundant,
    Compact,
}

impl Display for RowFormatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            RowFormatType::Default => write!(f, "ROW_FORMAT DEFAULT"),
            RowFormatType::Dynamic => write!(f, "ROW_FORMAT DYNAMIC"),
            RowFormatType::Fixed => write!(f, "ROW_FORMAT FIXED"),
            RowFormatType::Compressed => write!(f, "ROW_FORMAT COMPRESSED"),
            RowFormatType::Redundant => write!(f, "ROW_FORMAT REDUNDANT"),
            RowFormatType::Compact => write!(f, "ROW_FORMAT COMPACT"),
        }
    }
}

impl RowFormatType {
    pub fn parse(i: &str) -> IResult<&str, RowFormatType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ROW_FORMAT"),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((
                    map(tag_no_case("DEFAULT"), |_| RowFormatType::Default),
                    map(tag_no_case("DYNAMIC"), |_| RowFormatType::Dynamic),
                    map(tag_no_case("FIXED"), |_| RowFormatType::Fixed),
                    map(tag_no_case("COMPRESSED"), |_| RowFormatType::Compressed),
                    map(tag_no_case("REDUNDANT"), |_| RowFormatType::Redundant),
                    map(tag_no_case("COMPACT"), |_| RowFormatType::Compact),
                )),
            )),
            |(_, _, _, _, row_format_type)| row_format_type,
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use base::RowFormatType;

    #[test]
    fn parse_row_format_type() {
        let str1 = "ROW_FORMAT=FIXED";
        let res1 = RowFormatType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, RowFormatType::Fixed);
    }
}

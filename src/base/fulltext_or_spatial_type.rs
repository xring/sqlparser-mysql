use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {FULLTEXT | SPATIAL}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FulltextOrSpatialType {
    Fulltext,
    Spatial,
}

impl FulltextOrSpatialType {
    /// {FULLTEXT | SPATIAL}
    pub fn parse(i: &str) -> IResult<&str, FulltextOrSpatialType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("FULLTEXT"), |_| FulltextOrSpatialType::Fulltext),
            map(tag_no_case("SPATIAL"), |_| FulltextOrSpatialType::Spatial),
        ))(i)
    }
}

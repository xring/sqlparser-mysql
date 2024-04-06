use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;

use base::ParseSQLError;

/// `[MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Full,
    Partial,
    Simple,
}

impl MatchType {
    /// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
    pub fn parse(i: &str) -> IResult<&str, MatchType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MATCH"),
                multispace1,
                alt((
                    map(tag_no_case("FULL"), |_| MatchType::Full),
                    map(tag_no_case("PARTIAL"), |_| MatchType::Partial),
                    map(tag_no_case("SIMPLE"), |_| MatchType::Simple),
                )),
            )),
            |x| x.2,
        )(i)
    }
}

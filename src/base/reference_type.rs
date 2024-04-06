use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;

use base::ParseSQLError;

/// reference_option:
///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReferenceType {
    Restrict,
    Cascade,
    SetNull,
    NoAction,
    SetDefault,
}

impl ReferenceType {
    /// reference_option:
    ///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
    pub fn parse(i: &str) -> IResult<&str, ReferenceType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("RESTRICT"), |_| ReferenceType::Restrict),
            map(tag_no_case("CASCADE"), |_| ReferenceType::Cascade),
            map(
                tuple((tag_no_case("SET"), multispace1, tag_no_case("NULL"))),
                |_| ReferenceType::SetNull,
            ),
            map(
                tuple((tag_no_case("NO"), multispace1, tag_no_case("ACTION"))),
                |_| ReferenceType::NoAction,
            ),
            map(
                tuple((tag_no_case("SET"), multispace1, tag_no_case("DEFAULT"))),
                |_| ReferenceType::SetDefault,
            ),
        ))(i)
    }
}

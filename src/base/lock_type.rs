use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::ParseSQLError;

/// LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LockType {
    Default,
    None,
    Shared,
    Exclusive,
}

impl LockType {
    /// lock_option:
    ///     LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
    pub fn parse(i: &str) -> IResult<&str, LockType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("LOCK "),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((
                    map(tag_no_case("DEFAULT"), |_| LockType::Default),
                    map(tag_no_case("NONE"), |_| LockType::None),
                    map(tag_no_case("SHARED"), |_| LockType::Shared),
                    map(tag_no_case("EXCLUSIVE"), |_| LockType::Exclusive),
                )),
                multispace0,
            )),
            |x| x.4,
        )(i)
    }
}

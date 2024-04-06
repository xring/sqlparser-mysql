use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::ParseSQLError;

/// ALGORITHM [=] {DEFAULT | INSTANT | INPLACE | COPY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlgorithmType {
    Instant, // alter table only
    Default,
    Inplace,
    Copy,
}

impl AlgorithmType {
    /// algorithm_option:
    ///     ALGORITHM [=] {DEFAULT | INSTANT | INPLACE | COPY}
    pub fn parse(i: &str) -> IResult<&str, AlgorithmType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ALGORITHM"),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((
                    map(tag_no_case("DEFAULT"), |_| AlgorithmType::Default),
                    map(tag_no_case("INSTANT"), |_| AlgorithmType::Instant),
                    map(tag_no_case("INPLACE"), |_| AlgorithmType::Inplace),
                    map(tag_no_case("COPY"), |_| AlgorithmType::Copy),
                )),
            )),
            |x| x.4,
        )(i)
    }
}

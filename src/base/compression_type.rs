use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::map;
use nom::sequence::delimited;
use nom::IResult;

use base::ParseSQLError;

/// {'ZLIB' | 'LZ4' | 'NONE'}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    ZLIB,
    LZ4,
    NONE,
}

impl CompressionType {
    pub fn parse(i: &str) -> IResult<&str, CompressionType, ParseSQLError<&str>> {
        alt((
            map(
                delimited(
                    alt((tag("'"), tag("\""))),
                    tag_no_case("ZLIB"),
                    alt((tag("'"), tag("\""))),
                ),
                |_| CompressionType::ZLIB,
            ),
            map(
                delimited(
                    alt((tag("'"), tag("\""))),
                    tag_no_case("LZ4"),
                    alt((tag("'"), tag("\""))),
                ),
                |_| CompressionType::LZ4,
            ),
            map(
                delimited(
                    alt((tag("'"), tag("\""))),
                    tag_no_case("NONE"),
                    alt((tag("'"), tag("\""))),
                ),
                |_| CompressionType::NONE,
            ),
        ))(i)
    }
}

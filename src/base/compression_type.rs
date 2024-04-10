use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, tuple};
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// parse `COMPRESSION [=] {'ZLIB' | 'LZ4' | 'NONE'}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    ZLIB,
    LZ4,
    NONE,
}

impl Display for CompressionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            CompressionType::ZLIB => write!(f, "COMPRESSION 'ZLIB'"),
            CompressionType::LZ4 => write!(f, "COMPRESSION 'LZ4'"),
            CompressionType::NONE => write!(f, "COMPRESSION 'NONE'"),
        }
    }
}

impl CompressionType {
    pub fn parse(i: &str) -> IResult<&str, CompressionType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("COMPRESSION"),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((
                    map(
                        alt((tag_no_case("'ZLIB'"), tag_no_case("\"ZLIB\""))),
                        |_| CompressionType::ZLIB,
                    ),
                    map(alt((tag_no_case("'LZ4'"), tag_no_case("\"LZ4\""))), |_| {
                        CompressionType::LZ4
                    }),
                    map(
                        alt((tag_no_case("'NONE'"), tag_no_case("\"NONE\""))),
                        |_| CompressionType::NONE,
                    ),
                )),
            )),
            |(_, _, _, _, compression_type)| compression_type,
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use base::CompressionType;

    #[test]
    fn parse_compression_type() {
        let str1 = "COMPRESSION 'ZLIB'";
        let res1 = CompressionType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, CompressionType::ZLIB);
    }
}

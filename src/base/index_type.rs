use core::fmt;
use std::fmt::Formatter;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, tuple};
use nom::IResult;

use base::ParseSQLError;

/// parse `USING {BTREE | HASH}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexType {
    Btree,
    Hash,
}

impl IndexType {
    pub fn parse(i: &str) -> IResult<&str, IndexType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("USING"),
                multispace1,
                alt((
                    map(tag_no_case("BTREE"), |_| IndexType::Btree),
                    map(tag_no_case("HASH"), |_| IndexType::Hash),
                )),
            )),
            |x| x.2,
        )(i)
    }

    /// `[index_type]`
    /// USING {BTREE | HASH}
    pub fn opt_index_type(i: &str) -> IResult<&str, Option<IndexType>, ParseSQLError<&str>> {
        opt(map(
            delimited(multispace1, IndexType::parse, multispace0),
            |x| x,
        ))(i)
    }
}

impl fmt::Display for IndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::Btree => write!(f, "USING BTREE")?,
            IndexType::Hash => write!(f, "USING HASH")?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::index_type::IndexType;

    #[test]
    fn parse_index_type() {
        let str1 = "using   hash";
        let res1 = IndexType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, IndexType::Hash);

        let str2 = "USING btree   ";
        let res2 = IndexType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, IndexType::Btree);
    }
}

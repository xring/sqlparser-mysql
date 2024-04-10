use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// parse `[MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Full,
    Partial,
    Simple,
}

impl Display for MatchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            MatchType::Full => write!(f, "MATCH FULL"),
            MatchType::Partial => write!(f, "MATCH PARTIAL"),
            MatchType::Simple => write!(f, "MATCH SIMPLE"),
        }
    }
}

impl MatchType {
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

#[cfg(test)]
mod tests {
    use base::MatchType;

    #[test]
    fn parse_algorithm_type() {
        let str1 = "MATCH Full ";
        let res1 = MatchType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, MatchType::Full);

        let str2 = "match PARTIAL";
        let res2 = MatchType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, MatchType::Partial);

        let str3 = "match  SIMPLE   ";
        let res3 = MatchType::parse(str3);
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, MatchType::Simple);
    }
}

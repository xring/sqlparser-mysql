use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// reference_option:
///     `RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReferenceType {
    Restrict,
    Cascade,
    SetNull,
    NoAction,
    SetDefault,
}

impl Display for ReferenceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ReferenceType::Restrict => write!(f, "RESTRICT"),
            ReferenceType::Cascade => write!(f, "CASCADE"),
            ReferenceType::SetNull => write!(f, "SET NULL"),
            ReferenceType::NoAction => write!(f, "NO ACTION"),
            ReferenceType::SetDefault => write!(f, "SET DEFAULT"),
        }
    }
}

impl ReferenceType {
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

#[cfg(test)]
mod tests {
    use base::reference_type::ReferenceType;

    #[test]
    fn parse_algorithm_type() {
        let str1 = "RESTRICT";
        let res1 = ReferenceType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, ReferenceType::Restrict);

        let str2 = "SET  NULL  ";
        let res2 = ReferenceType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, ReferenceType::SetNull);

        let str3 = "SET DEFAULT";
        let res3 = ReferenceType::parse(str3);
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, ReferenceType::SetDefault);
    }
}

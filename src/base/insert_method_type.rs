use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::streaming::tag;
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// parse `INSERT_METHOD [=] { NO | FIRST | LAST }`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InsertMethodType {
    No,
    First,
    Last,
}

impl Display for InsertMethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            InsertMethodType::No => write!(f, "INSERT_METHOD NO"),
            InsertMethodType::First => write!(f, "INSERT_METHOD FIRST"),
            InsertMethodType::Last => write!(f, "INSERT_METHOD LAST"),
        }
    }
}

impl InsertMethodType {
    pub fn parse(i: &str) -> IResult<&str, InsertMethodType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("INSERT_METHOD"),
                multispace0,
                opt(tag_no_case("=")),
                multispace0,
                alt((
                    map(tag_no_case("NO"), |_| InsertMethodType::No),
                    map(tag_no_case("FIRST"), |_| InsertMethodType::First),
                    map(tag_no_case("LAST"), |_| InsertMethodType::Last),
                )),
            )),
            |(_, _, _, _, inert_method_type)| inert_method_type,
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use base::InsertMethodType;

    #[test]
    fn parse_insert_method_type() {
        let str1 = "INSERT_METHOD NO";
        let res1 = InsertMethodType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, InsertMethodType::No);

        let str2 = "INSERT_METHOD=NO";
        let res2 = InsertMethodType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, InsertMethodType::No);

        let str3 = "INSERT_METHOD= FIRST";
        let res3 = InsertMethodType::parse(str3);
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, InsertMethodType::First);

        let str4 = "INSERT_METHOD =FIRST";
        let res4 = InsertMethodType::parse(str4);
        assert!(res4.is_ok());
        assert_eq!(res4.unwrap().1, InsertMethodType::First);

        let str5 = "INSERT_METHOD = LAST";
        let res5 = InsertMethodType::parse(str5);
        assert!(res5.is_ok());
        assert_eq!(res5.unwrap().1, InsertMethodType::Last);
    }
}

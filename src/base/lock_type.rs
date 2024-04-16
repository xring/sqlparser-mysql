use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// lock_option:
///     parse `LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LockType {
    Default,
    None,
    Shared,
    Exclusive,
}

impl Display for LockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            LockType::Default => write!(f, "LOCK DEFAULT"),
            LockType::None => write!(f, "LOCK NONE"),
            LockType::Shared => write!(f, "LOCK SHARED"),
            LockType::Exclusive => write!(f, "LOCK EXCLUSIVE"),
        }
    }
}

impl LockType {
    pub fn parse(i: &str) -> IResult<&str, LockType, ParseSQLError<&str>> {
        alt((
            map(
                tuple((tag_no_case("LOCK"), multispace1, Self::parse_lock)),
                |(_, _, lock)| lock,
            ),
            map(
                tuple((
                    tag_no_case("LOCK"),
                    multispace0,
                    tag("="),
                    multispace0,
                    Self::parse_lock,
                )),
                |(_, _, _, _, lock)| lock,
            ),
        ))(i)
    }

    fn parse_lock(i: &str) -> IResult<&str, LockType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DEFAULT"), |_| LockType::Default),
            map(tag_no_case("NONE"), |_| LockType::None),
            map(tag_no_case("SHARED"), |_| LockType::Shared),
            map(tag_no_case("EXCLUSIVE"), |_| LockType::Exclusive),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use base::lock_type::LockType;

    #[test]
    fn parse_lock_type() {
        let str1 = "LOCK EXCLUSIVE";
        let res1 = LockType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, LockType::Exclusive);

        let str2 = "lock=DEFAULT";
        let res2 = LockType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, LockType::Default);

        let str3 = "LOCK= NONE";
        let res3 = LockType::parse(str3);
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, LockType::None);

        let str4 = "lock =SHARED";
        let res4 = LockType::parse(str4);
        assert!(res4.is_ok());
        assert_eq!(res4.unwrap().1, LockType::Shared);

        let str5 = "lockSHARED";
        let res5 = LockType::parse(str5);
        assert!(res5.is_err());
    }
}

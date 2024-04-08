use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::ParseSQLError;

/// lock_option:
///     LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LockType {
    Default,
    None,
    Shared,
    Exclusive,
}

impl LockType {
    /// parse `LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}`
    pub fn parse(i: &str) -> IResult<&str, LockType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("LOCK"),
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
    }
}
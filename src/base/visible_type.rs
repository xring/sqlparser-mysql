use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {VISIBLE | INVISIBLE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum VisibleType {
    Visible,
    Invisible,
}

impl VisibleType {
    pub fn parse(i: &str) -> IResult<&str, VisibleType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("VISIBLE"), |_| VisibleType::Visible),
            map(tag_no_case("INVISIBLE"), |_| VisibleType::Invisible),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use base::visible_type::VisibleType;

    #[test]
    fn parse_algorithm_type() {
        let str1 = "visible";
        let res1 = VisibleType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, VisibleType::Visible);

        let str2 = "invisible";
        let res2 = VisibleType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, VisibleType::Invisible);
    }
}

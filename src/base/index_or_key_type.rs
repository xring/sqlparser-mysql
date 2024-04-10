use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;
use std::fmt::{Display, Formatter};

use base::ParseSQLError;

/// parse `{INDEX | KEY}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexOrKeyType {
    Index,
    Key,
}

impl Display for IndexOrKeyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            IndexOrKeyType::Index => write!(f, "INDEX"),
            IndexOrKeyType::Key => write!(f, "KEY"),
        }
    }
}

impl IndexOrKeyType {
    /// {INDEX | KEY}
    pub fn parse(i: &str) -> IResult<&str, IndexOrKeyType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("KEY"), |_| IndexOrKeyType::Key),
            map(tag_no_case("INDEX"), |_| IndexOrKeyType::Index),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use base::index_or_key_type::IndexOrKeyType;

    #[test]
    fn parse_index_or_key_type() {
        let str1 = "index";
        let res1 = IndexOrKeyType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, IndexOrKeyType::Index);

        let str2 = "KEY";
        let res2 = IndexOrKeyType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, IndexOrKeyType::Key);
    }
}

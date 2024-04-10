use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::{write, Display, Formatter};

use base::ParseSQLError;

/// STORAGE {DISK | MEMORY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TablespaceType {
    StorageDisk,
    StorageMemory,
}

impl Display for TablespaceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            TablespaceType::StorageDisk => write!(f, "STORAGE DISK"),
            TablespaceType::StorageMemory => write!(f, "STORAGE MEMORY"),
        }
    }
}

impl TablespaceType {
    pub fn parse(i: &str) -> IResult<&str, TablespaceType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STORAGE"),
                multispace0,
                alt((
                    map(tag_no_case("DISK"), |_| TablespaceType::StorageDisk),
                    map(tag_no_case("MEMORY"), |_| TablespaceType::StorageMemory),
                )),
            )),
            |(_, _, tablespace_type)| tablespace_type,
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use base::algorithm_type::AlgorithmType;
    use base::TablespaceType;

    #[test]
    fn parse_algorithm_type() {
        let str1 = "STORAGE disk";
        let res1 = TablespaceType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, TablespaceType::StorageDisk);

        let str2 = "storage   memory";
        let res2 = TablespaceType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, TablespaceType::StorageMemory);
    }
}

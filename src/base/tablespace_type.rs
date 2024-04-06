use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// STORAGE {DISK | MEMORY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TablespaceType {
    StorageDisk,
    StorageMemory,
}

impl TablespaceType {
    pub fn parse(i: &str) -> IResult<&str, TablespaceType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DISK"), |_| TablespaceType::StorageDisk),
            map(tag_no_case("MEMORY"), |_| TablespaceType::StorageMemory),
        ))(i)
    }
}

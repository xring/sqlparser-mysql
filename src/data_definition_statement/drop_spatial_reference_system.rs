use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use common_parsers::{parse_if_exists, statement_terminator};
use data_definition_statement;

/// DROP SPATIAL REFERENCE SYSTEM
///     [IF EXISTS]
///     srid
///
/// srid: 32-bit unsigned integer
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropSpatialReferenceSystemStatement {
    pub if_exists: bool,
    pub srid: u32,
}

impl fmt::Display for DropSpatialReferenceSystemStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP SPATIAL REFERENCE SYSTEM")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }
        write!(f, " {}", self.srid)?;
        Ok(())
    }
}

/// DROP SPATIAL REFERENCE SYSTEM
///     [IF EXISTS]
///     srid
///
/// srid: 32-bit unsigned integer
pub fn drop_spatial_reference_system_parser(
    i: &str,
) -> IResult<&str, DropSpatialReferenceSystemStatement> {
    map(
        tuple((
            terminated(tag_no_case("DROP"), multispace1),
            terminated(tag_no_case("SPATIAL"), multispace1),
            terminated(tag_no_case("REFERENCE"), multispace1),
            terminated(tag_no_case("SYSTEM"), multispace1),
            parse_if_exists,
            complete::u32,
            multispace0,
            statement_terminator,
        )),
        |x| DropSpatialReferenceSystemStatement {
            if_exists: x.4.is_some(),
            srid: x.5,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use data_definition_statement::drop_spatial_reference_system::drop_spatial_reference_system_parser;

    #[test]
    fn test_drop_spatial_reference_system() {
        let sqls = vec![
            "DROP SPATIAL REFERENCE SYSTEM 4120;",
            "DROP SPATIAL REFERENCE SYSTEM IF EXISTS 4120;",
        ];
        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_spatial_reference_system_parser(sqls[i]);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}

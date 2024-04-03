use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::error::VerboseError;
use nom::IResult;
use nom::sequence::{terminated, tuple};

use common::{parse_if_exists, sql_identifier, statement_terminator};

/// DROP FUNCTION [IF EXISTS] sp_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropFunctionStatement {
    pub if_exists: bool,
    pub sp_name: String,
}

impl DropFunctionStatement {
    /// DROP FUNCTION [IF EXISTS] sp_name
    pub fn parse(i: &str) -> IResult<&str, DropFunctionStatement, VerboseError<&str>> {
        map(
            tuple((
                terminated(tag_no_case("DROP"), multispace1),
                terminated(tag_no_case("FUNCTION"), multispace1),
                parse_if_exists,
                map(sql_identifier, |sp_name| String::from(sp_name)),
                multispace0,
                statement_terminator,
            )),
            |x| DropFunctionStatement {
                if_exists: x.2.is_some(),
                sp_name: x.3,
            },
        )(i)
    }
}

impl fmt::Display for DropFunctionStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP FUNCTION")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }
        write!(f, " {}", self.sp_name)?;
        Ok(())
    }
}

/// DROP FUNCTION [IF EXISTS] sp_name
pub fn drop_function(i: &str) -> IResult<&str, DropFunctionStatement, VerboseError<&str>> {
    map(
        tuple((
            terminated(tag_no_case("DROP"), multispace1),
            terminated(tag_no_case("FUNCTION"), multispace1),
            parse_if_exists,
            map(sql_identifier, |sp_name| String::from(sp_name)),
            multispace0,
            statement_terminator,
        )),
        |x| DropFunctionStatement {
            if_exists: x.2.is_some(),
            sp_name: x.3,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use dds::drop_function::DropFunctionStatement;

    #[test]
    fn test_drop_function() {
        let sqls = vec!["DROP FUNCTION sp_name;", "DROP FUNCTION IF EXISTS sp_name;"];
        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = DropFunctionStatement::parse(sqls[i]);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}

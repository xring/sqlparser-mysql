use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP FUNCTION [IF EXISTS] sp_name`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropFunctionStatement {
    pub if_exists: bool,
    pub sp_name: String,
}

impl DropFunctionStatement {
    pub fn parse(i: &str) -> IResult<&str, DropFunctionStatement, ParseSQLError<&str>> {
        map(
            tuple((
                terminated(tag_no_case("DROP"), multispace1),
                terminated(tag_no_case("FUNCTION"), multispace1),
                CommonParser::parse_if_exists,
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                CommonParser::statement_terminator,
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

#[cfg(test)]
mod tests {
    use dds::drop_function::DropFunctionStatement;

    #[test]
    fn parse_drop_function() {
        let sqls = ["DROP FUNCTION sp_name;", "DROP FUNCTION IF EXISTS sp_name;"];
        let exp_statements = [
            DropFunctionStatement {
                if_exists: false,
                sp_name: "sp_name".to_string(),
            },
            DropFunctionStatement {
                if_exists: true,
                sp_name: "sp_name".to_string(),
            },
        ];

        for i in 0..sqls.len() {
            let res = DropFunctionStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i])
        }
    }
}

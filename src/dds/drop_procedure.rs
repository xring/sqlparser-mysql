use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP PROCEDURE [IF EXISTS] sp_name`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropProcedureStatement {
    pub if_exists: bool,
    pub sp_name: String,
}

impl DropProcedureStatement {
    pub fn parse(i: &str) -> IResult<&str, DropProcedureStatement, ParseSQLError<&str>> {
        map(
            tuple((
                terminated(tag_no_case("DROP"), multispace1),
                terminated(tag_no_case("PROCEDURE"), multispace1),
                CommonParser::parse_if_exists,
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                CommonParser::statement_terminator,
            )),
            |x| DropProcedureStatement {
                if_exists: x.2.is_some(),
                sp_name: x.3,
            },
        )(i)
    }
}

impl fmt::Display for DropProcedureStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP PROCEDURE")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }
        write!(f, " {}", self.sp_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_procedure::DropProcedureStatement;

    #[test]
    fn parse_drop_procedure() {
        let sqls = [
            "DROP PROCEDURE sp_name;",
            "DROP PROCEDURE IF EXISTS sp_name;",
        ];
        let exp_statements = [
            DropProcedureStatement {
                if_exists: false,
                sp_name: "sp_name".to_string(),
            },
            DropProcedureStatement {
                if_exists: true,
                sp_name: "sp_name".to_string(),
            },
        ];
        for i in 0..sqls.len() {
            let res = DropProcedureStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i]);
        }
    }
}

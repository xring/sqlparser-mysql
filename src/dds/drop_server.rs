use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP SERVER [ IF EXISTS ] server_name`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropServerStatement {
    pub if_exists: bool,
    pub server_name: String,
}

impl DropServerStatement {
    /// DROP SERVER [ IF EXISTS ] server_name
    pub fn parse(i: &str) -> IResult<&str, DropServerStatement, ParseSQLError<&str>> {
        map(
            tuple((
                terminated(tag_no_case("DROP"), multispace1),
                terminated(tag_no_case("SERVER"), multispace1),
                CommonParser::parse_if_exists,
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                CommonParser::statement_terminator,
            )),
            |x| DropServerStatement {
                if_exists: x.2.is_some(),
                server_name: x.3,
            },
        )(i)
    }
}

impl fmt::Display for DropServerStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP SERVER")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }
        write!(f, " {}", self.server_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_server::DropServerStatement;

    #[test]
    fn test_drop_server() {
        let sqls = [
            "DROP SERVER server_name;",
            "DROP SERVER IF EXISTS server_name;",
        ];
        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = DropServerStatement::parse(sqls[i]);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}

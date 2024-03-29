use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use common_parsers::{parse_if_exists, sql_identifier, statement_terminator};
use data_definition_statement;

/// DROP SERVER [ IF EXISTS ] server_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropServerStatement {
    pub if_exists: bool,
    pub server_name: String,
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

/// DROP SERVER [ IF EXISTS ] server_name
pub fn drop_server_parser(i: &str) -> IResult<&str, DropServerStatement> {
    map(
        tuple((
            terminated(tag_no_case("DROP"), multispace1),
            terminated(tag_no_case("SERVER"), multispace1),
            parse_if_exists,
            map(sql_identifier, |server_name| String::from(server_name)),
            multispace0,
            statement_terminator,
        )),
        |x| DropServerStatement {
            if_exists: x.2.is_some(),
            server_name: x.3,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use data_definition_statement::drop_server_parser;

    #[test]
    fn test_drop_server() {
        let sqls = vec![
            "DROP SERVER server_name;",
            "DROP SERVER IF EXISTS server_name;",
        ];
        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_server_parser(sqls[i]);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}

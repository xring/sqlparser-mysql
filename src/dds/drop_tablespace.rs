use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP [UNDO] TABLESPACE tablespace_name
///     [ENGINE [=] engine_name]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropTablespaceStatement {
    pub undo: bool,
    pub tablespace_name: String,
    pub engine_name: Option<String>,
}

impl DropTablespaceStatement {
    pub fn parse(i: &str) -> IResult<&str, DropTablespaceStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP "),
            multispace0,
            opt(tag_no_case("UNDO")),
            multispace0,
            tag_no_case("TABLESPACE "),
            multispace0,
            map(CommonParser::sql_identifier, |tablespace_name| {
                String::from(tablespace_name)
            }),
            multispace0,
            opt(map(
                tuple((
                    tag_no_case("ENGINE"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                    CommonParser::sql_identifier,
                    multispace0,
                )),
                |(_, _, _, _, engine, _)| String::from(engine),
            )),
            multispace0,
            CommonParser::statement_terminator,
        ));
        let (remaining_input, (_, _, opt_undo, _, _, _, tablespace_name, _, engine_name, _, _)) =
            parser(i)?;

        Ok((
            remaining_input,
            DropTablespaceStatement {
                undo: opt_undo.is_some(),
                tablespace_name,
                engine_name,
            },
        ))
    }
}

impl fmt::Display for DropTablespaceStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP")?;
        if self.undo {
            write!(f, " UNDO")?;
        }
        write!(f, " TABLESPACE")?;
        write!(f, " {}", self.tablespace_name)?;
        if let Some(ref engine_name) = self.engine_name {
            write!(f, " ENGINE = {}", engine_name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_tablespace::DropTablespaceStatement;

    #[test]
    fn parse_drop_tablespace() {
        let sqls = [
            "DROP TABLESPACE tablespace_name;",
            "DROP UNDO TABLESPACE tablespace_name;",
            "DROP TABLESPACE tablespace_name ENGINE = demo;",
            "DROP UNDO TABLESPACE tablespace_name ENGINE = demo;",
        ];

        let exp_statements = [
            DropTablespaceStatement {
                undo: false,
                tablespace_name: "tablespace_name".to_string(),
                engine_name: None,
            },
            DropTablespaceStatement {
                undo: true,
                tablespace_name: "tablespace_name".to_string(),
                engine_name: None,
            },
            DropTablespaceStatement {
                undo: false,
                tablespace_name: "tablespace_name".to_string(),
                engine_name: Some("demo".to_string()),
            },
            DropTablespaceStatement {
                undo: true,
                tablespace_name: "tablespace_name".to_string(),
                engine_name: Some("demo".to_string()),
            },
        ];

        for i in 0..sqls.len() {
            let res = DropTablespaceStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i])
        }
    }
}

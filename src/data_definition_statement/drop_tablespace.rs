use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::IResult;
use nom::sequence::tuple;

use common_parsers::{
    sql_identifier,
    statement_terminator,
};

/// DROP [UNDO] TABLESPACE tablespace_name
///     [ENGINE [=] engine_name]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropTablespaceStatement {
    pub undo: bool,
    pub tablespace_name: String,
    pub engine_name: Option<String>,
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

/// DROP [UNDO] TABLESPACE tablespace_name
///     [ENGINE [=] engine_name]
pub fn drop_tablespace_parser(i: &[u8]) -> IResult<&[u8], DropTablespaceStatement> {
    let mut parser = tuple((
        tag_no_case("DROP "),
        multispace0,
        opt(tag_no_case("UNDO")),
        multispace0,
        tag_no_case("TABLESPACE "),
        multispace0,
        map(sql_identifier, |tablespace_name| {
            String::from_utf8(tablespace_name.to_vec()).unwrap()
        }),
        multispace0,
        opt(map(
            tuple((
                tag_no_case("ENGINE"),
                multispace1,
                opt(tag("=")),
                multispace0,
                sql_identifier,
                multispace0,
            )),
            |(_, _, _, _, engine, _)| String::from_utf8(engine.to_vec()).unwrap(),
        )),
        multispace0,
        statement_terminator,
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

#[cfg(test)]
mod tests {
    use data_definition_statement::drop_tablespace::drop_tablespace_parser;

    #[test]
    fn test_drop_tablespace_parser() {
        let sqls = vec![
            "DROP TABLESPACE tablespace_name;",
            "DROP UNDO TABLESPACE tablespace_name;",
            "DROP TABLESPACE tablespace_name ENGINE = demo;",
            "DROP UNDO TABLESPACE tablespace_name ENGINE = demo;",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_tablespace_parser(sqls[i].as_bytes());
            // res.unwrap();
            println!("{:?}", res);
            // assert!(res.is_ok());
            // println!("{:?}", res);
        }
    }
}

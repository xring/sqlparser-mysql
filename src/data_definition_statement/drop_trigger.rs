use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use common::table::Table;
use common::trigger::Trigger;
use common_parsers::{
    parse_if_exists, schema_table_name_without_alias, schema_trigger_name, statement_terminator,
    ws_sep_comma,
};

/// DROP TRIGGER [IF EXISTS] [schema_name.]trigger_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropTriggerStatement {
    pub if_exists: bool,
    pub trigger_name: Trigger,
}

impl fmt::Display for DropTriggerStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP TRIGGER")?;
        if self.if_exists {
            write!(f, " IF EXISTS ")?;
        }
        write!(f, "{}", self.trigger_name)?;
        Ok(())
    }
}

/// DROP TRIGGER [IF EXISTS] [schema_name.]trigger_name
pub fn drop_trigger_parser(i: &[u8]) -> IResult<&[u8], DropTriggerStatement> {
    let mut parser = tuple((
        tag_no_case("DROP "),
        multispace0,
        tag_no_case("TRIGGER "),
        parse_if_exists,
        multispace0,
        schema_trigger_name,
        multispace0,
        statement_terminator,
    ));
    let (remaining_input, (_, _, _, opt_if_exists, _, trigger_name, _, _)) = parser(i)?;

    Ok((
        remaining_input,
        DropTriggerStatement {
            if_exists: opt_if_exists.is_some(),
            trigger_name,
        },
    ))
}

#[cfg(test)]
mod tests {
    use data_definition_statement::drop_trigger::drop_trigger_parser;

    #[test]
    fn test_drop_trigger() {
        let sqls = vec![
            "DROP TRIGGER trigger_name;",
            "DROP TRIGGER db_name.trigger_name;",
            "DROP TRIGGER IF EXISTS trigger_name;",
            "DROP TRIGGER IF EXISTS db_name.trigger_name;",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_trigger_parser(sqls[i].as_bytes());
            // res.unwrap();
            println!("{:?}", res);
            // assert!(res.is_ok());
            // println!("{:?}", res);
        }
    }
}

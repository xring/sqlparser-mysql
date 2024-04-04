use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::trigger::Trigger;
use common::{parse_if_exists, statement_terminator};

/// DROP TRIGGER [IF EXISTS] [schema_name.]trigger_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropTriggerStatement {
    pub if_exists: bool,
    pub trigger_name: Trigger,
}

impl DropTriggerStatement {
    /// DROP TRIGGER [IF EXISTS] [schema_name.]trigger_name
    pub fn parse(i: &str) -> IResult<&str, DropTriggerStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP "),
            multispace0,
            tag_no_case("TRIGGER "),
            parse_if_exists,
            multispace0,
            Trigger::parse,
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

#[cfg(test)]
mod tests {
    use dds::drop_trigger::DropTriggerStatement;

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
            let res = DropTriggerStatement::parse(sqls[i]);
            // res.unwrap();
            println!("{:?}", res);
            // assert!(res.is_ok());
            // println!("{:?}", res);
        }
    }
}

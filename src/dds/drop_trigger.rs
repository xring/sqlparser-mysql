use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::trigger::Trigger;
use base::CommonParser;

/// parse `DROP TRIGGER [IF EXISTS] [schema_name.]trigger_name`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropTriggerStatement {
    pub if_exists: bool,
    pub trigger_name: Trigger,
}

impl DropTriggerStatement {
    pub fn parse(i: &str) -> IResult<&str, DropTriggerStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP "),
            multispace0,
            tag_no_case("TRIGGER "),
            CommonParser::parse_if_exists,
            multispace0,
            Trigger::parse,
            multispace0,
            CommonParser::statement_terminator,
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
            write!(f, " IF EXISTS")?;
        }
        write!(f, " {}", self.trigger_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::Trigger;
    use dds::drop_trigger::DropTriggerStatement;

    #[test]
    fn test_drop_trigger() {
        let sqls = [
            "DROP TRIGGER trigger_name;",
            "DROP TRIGGER db_name.trigger_name;",
            "DROP TRIGGER IF EXISTS trigger_name;",
            "DROP TRIGGER IF EXISTS db_name.trigger_name;",
        ];

        let exp_statements = [
            DropTriggerStatement {
                if_exists: false,
                trigger_name: Trigger {
                    name: "trigger_name".to_string(),
                    schema: None,
                },
            },
            DropTriggerStatement {
                if_exists: false,
                trigger_name: Trigger {
                    name: "trigger_name".to_string(),
                    schema: Some("db_name".to_string()),
                },
            },
            DropTriggerStatement {
                if_exists: true,
                trigger_name: Trigger {
                    name: "trigger_name".to_string(),
                    schema: None,
                },
            },
            DropTriggerStatement {
                if_exists: true,
                trigger_name: Trigger {
                    name: "trigger_name".to_string(),
                    schema: Some("db_name".to_string()),
                },
            },
        ];

        for i in 0..sqls.len() {
            let res = DropTriggerStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i])
        }
    }
}

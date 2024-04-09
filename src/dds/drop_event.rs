use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP EVENT [IF EXISTS] event_name`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropEventStatement {
    pub if_exists: bool,
    pub event_name: String,
}

impl DropEventStatement {
    pub fn parse(i: &str) -> IResult<&str, DropEventStatement, ParseSQLError<&str>> {
        map(
            tuple((
                terminated(tag_no_case("DROP"), multispace1),
                terminated(tag_no_case("EVENT"), multispace1),
                CommonParser::parse_if_exists,
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                CommonParser::statement_terminator,
            )),
            |x| DropEventStatement {
                if_exists: x.2.is_some(),
                event_name: x.3,
            },
        )(i)
    }
}

impl fmt::Display for DropEventStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP EVENT")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }
        write!(f, " {}", self.event_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_event::DropEventStatement;

    #[test]
    fn parse_drop_event() {
        let sqls = ["DROP EVENT event_name;", "DROP EVENT IF EXISTS event_name;"];
        let exp_statements = [
            DropEventStatement {
                if_exists: false,
                event_name: "event_name".to_string(),
            },
            DropEventStatement {
                if_exists: true,
                event_name: "event_name".to_string(),
            },
        ];

        for i in 0..sqls.len() {
            let res = DropEventStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i]);
        }
    }
}

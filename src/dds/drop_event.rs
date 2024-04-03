use core::fmt;
use std::fmt::Formatter;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::error::VerboseError;
use nom::IResult;
use nom::sequence::{terminated, tuple};

use common::{parse_if_exists, sql_identifier, statement_terminator};

/// DROP EVENT [IF EXISTS] event_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropEventStatement {
    pub if_exists: bool,
    pub event_name: String,
}

impl DropEventStatement {
    /// DROP EVENT [IF EXISTS] event_name
    pub fn parse(i: &str) -> IResult<&str, DropEventStatement, VerboseError<&str>> {
        map(
            tuple((
                terminated(tag_no_case("DROP"), multispace1),
                terminated(tag_no_case("EVENT"), multispace1),
                parse_if_exists,
                map(sql_identifier, |event_name| String::from(event_name)),
                multispace0,
                statement_terminator,
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
    fn test_drop_event() {
        let sqls = vec!["DROP EVENT event_name;", "DROP EVENT IF EXISTS event_name;"];
        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = DropEventStatement::parse(sqls[i]);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}

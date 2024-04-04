use std::fmt;
use std::str;

use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::{pair, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use common::keywords::escape_if_keyword;
use common::sql_identifier;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub schema: Option<String>,
}

impl Trigger {
    pub fn parse(i: &str) -> IResult<&str, Trigger, ParseSQLError<&str>> {
        map(
            tuple((opt(pair(sql_identifier, tag("."))), sql_identifier)),
            |tup| Trigger {
                name: String::from(tup.1),
                schema: match tup.0 {
                    Some((schema, _)) => Some(String::from(schema)),
                    None => None,
                },
            },
        )(i)
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref schema) = self.schema {
            write!(f, "{}.", escape_if_keyword(schema))?;
        }
        write!(f, "{}", escape_if_keyword(&self.name))?;
        Ok(())
    }
}

impl<'a> From<&'a str> for Trigger {
    fn from(t: &str) -> Trigger {
        Trigger {
            name: String::from(t),
            schema: None,
        }
    }
}

impl<'a> From<(&'a str, &'a str)> for Trigger {
    fn from(t: (&str, &str)) -> Trigger {
        Trigger {
            name: String::from(t.1),
            schema: Some(String::from(t.0)),
        }
    }
}

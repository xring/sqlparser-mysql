use std::fmt;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::multi::many0;
use nom::sequence::{pair, terminated, tuple};
use nom::IResult;

use common::keywords::escape_if_keyword;
use common::{as_alias, sql_identifier, ws_sep_comma};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub alias: Option<String>,
    pub schema: Option<String>,
}

impl Table {
    // Parse list of table names.
    // XXX(malte): add support for aliases
    pub fn table_list(i: &str) -> IResult<&str, Vec<Table>, VerboseError<&str>> {
        many0(terminated(Table::schema_table_reference, opt(ws_sep_comma)))(i)
    }

    // Parse a reference to a named schema.table, with an optional alias
    pub fn schema_table_reference(i: &str) -> IResult<&str, Table, VerboseError<&str>> {
        map(
            tuple((
                opt(pair(sql_identifier, tag("."))),
                sql_identifier,
                opt(as_alias),
            )),
            |tup| Table {
                name: String::from(tup.1),
                alias: match tup.2 {
                    Some(a) => Some(String::from(a)),
                    None => None,
                },
                schema: match tup.0 {
                    Some((schema, _)) => Some(String::from(schema)),
                    None => None,
                },
            },
        )(i)
    }

    // Parse a reference to a named table, with an optional alias
    pub fn table_reference(i: &str) -> IResult<&str, Table, VerboseError<&str>> {
        map(pair(sql_identifier, opt(as_alias)), |tup| Table {
            name: String::from(tup.0),
            alias: match tup.1 {
                Some(a) => Some(String::from(a)),
                None => None,
            },
            schema: None,
        })(i)
    }

    /// table alias not allowed in DROP/TRUNCATE/RENAME TABLE statement
    pub fn without_alias(i: &str) -> IResult<&str, Table, VerboseError<&str>> {
        map(
            tuple((opt(pair(sql_identifier, tag("."))), sql_identifier)),
            |tup| Table {
                name: String::from(tup.1),
                alias: None,
                schema: match tup.0 {
                    Some((schema, _)) => Some(String::from(schema)),
                    None => None,
                },
            },
        )(i)
    }

    /// db_name.tb_name TO db_name.tb_name
    pub fn schema_table_reference_to_schema_table_reference(
        i: &str,
    ) -> IResult<&str, (Table, Table), VerboseError<&str>> {
        map(
            tuple((
                Self::schema_table_reference,
                multispace0,
                tag_no_case("TO"),
                multispace1,
                Self::schema_table_reference,
            )),
            |(from, _, _, _, to)| (from, to),
        )(i)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref schema) = self.schema {
            write!(f, "{}.", escape_if_keyword(schema))?;
        }
        write!(f, "{}", escape_if_keyword(&self.name))?;
        if let Some(ref alias) = self.alias {
            write!(f, " AS {}", escape_if_keyword(alias))?;
        }
        Ok(())
    }
}

impl<'a> From<&'a str> for Table {
    fn from(t: &str) -> Table {
        Table {
            name: String::from(t),
            alias: None,
            schema: None,
        }
    }
}

impl<'a> From<(&'a str, &'a str)> for Table {
    fn from(t: (&str, &str)) -> Table {
        Table {
            name: String::from(t.1),
            alias: None,
            schema: Some(String::from(t.0)),
        }
    }
}

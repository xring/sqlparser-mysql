use std::fmt;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{pair, terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::{CommonParser, DisplayUtil};

/// **Table Definition**
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Optional table name alias
    pub alias: Option<String>,
    /// Optional schema/database name
    pub schema: Option<String>,
}

impl Table {
    // Parse list of table names.
    // XXX(malte): add support for aliases
    pub fn table_list(i: &str) -> IResult<&str, Vec<Table>, ParseSQLError<&str>> {
        many0(terminated(
            Table::schema_table_reference,
            opt(CommonParser::ws_sep_comma),
        ))(i)
    }

    // Parse a reference to a named schema.table, with an optional alias
    pub fn schema_table_reference(i: &str) -> IResult<&str, Table, ParseSQLError<&str>> {
        map(
            tuple((
                opt(pair(CommonParser::sql_identifier, tag("."))),
                CommonParser::sql_identifier,
                opt(CommonParser::as_alias),
            )),
            |tup| Table {
                name: String::from(tup.1),
                alias: tup.2.map(String::from),
                schema: tup.0.map(|(schema, _)| String::from(schema)),
            },
        )(i)
    }

    // Parse a reference to a named table, with an optional alias
    pub fn table_reference(i: &str) -> IResult<&str, Table, ParseSQLError<&str>> {
        map(
            pair(CommonParser::sql_identifier, opt(CommonParser::as_alias)),
            |tup| Table {
                name: String::from(tup.0),
                alias: tup.1.map(String::from),
                schema: None,
            },
        )(i)
    }

    /// table alias not allowed in DROP/TRUNCATE/RENAME TABLE statement
    pub fn without_alias(i: &str) -> IResult<&str, Table, ParseSQLError<&str>> {
        map(
            tuple((
                opt(pair(CommonParser::sql_identifier, tag("."))),
                CommonParser::sql_identifier,
            )),
            |tup| Table {
                name: String::from(tup.1),
                alias: None,
                schema: tup.0.map(|(schema, _)| String::from(schema)),
            },
        )(i)
    }

    /// db_name.tb_name TO db_name.tb_name
    pub fn schema_table_reference_to_schema_table_reference(
        i: &str,
    ) -> IResult<&str, (Table, Table), ParseSQLError<&str>> {
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
            write!(f, "{}.", DisplayUtil::escape_if_keyword(schema))?;
        }
        write!(f, "{}", DisplayUtil::escape_if_keyword(&self.name))?;
        if let Some(ref alias) = self.alias {
            write!(f, " AS {}", DisplayUtil::escape_if_keyword(alias))?;
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

#[cfg(test)]
mod tests {
    use base::Table;

    #[test]
    fn parse_trigger() {
        let str1 = "tbl_name";
        let res1 = Table::table_reference(str1);
        let exp1 = Table {
            name: "tbl_name".to_string(),
            alias: None,
            schema: None,
        };
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, exp1);

        let str2 = "foo.tbl_name";
        let res2 = Table::schema_table_reference(str2);
        let exp2 = Table {
            name: "tbl_name".to_string(),
            alias: None,
            schema: Some("foo".to_string()),
        };
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, exp2);

        let str3 = "foo.tbl_name as bar";
        let res3 = Table::schema_table_reference(str3);
        let exp3 = Table {
            name: "tbl_name".to_string(),
            alias: Some("bar".to_string()),
            schema: Some("foo".to_string()),
        };
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, exp3);
    }

    #[test]
    fn from_str() {
        let trigger1: Table = "tbl_name".into();
        let exp1 = Table {
            name: "tbl_name".to_string(),
            alias: None,
            schema: None,
        };
        assert_eq!(trigger1, exp1);
    }

    #[test]
    fn from_tuple_str() {
        let table2: Table = ("foo", "tbl_name").into();
        let exp2 = Table {
            name: "tbl_name".to_string(),
            alias: None,
            schema: Some("foo".to_string()),
        };
        assert_eq!(table2, exp2);
    }
}

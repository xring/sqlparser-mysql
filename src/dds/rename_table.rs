use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::table::Table;
use base::CommonParser;

/// parse `RENAME TABLE
///     tbl_name TO new_tbl_name
///     [, tbl_name2 TO new_tbl_name2] ...`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RenameTableStatement {
    pub tables: Vec<(Table, Table)>,
}

impl RenameTableStatement {
    pub fn parse(i: &str) -> IResult<&str, RenameTableStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("RENAME "),
            multispace0,
            tag_no_case("TABLE "),
            multispace0,
            many0(terminated(
                Table::schema_table_reference_to_schema_table_reference,
                opt(CommonParser::ws_sep_comma),
            )),
            CommonParser::statement_terminator,
        ));
        let (remaining_input, (_, _, _, _, table_pairs, _)) = parser(i)?;

        Ok((
            remaining_input,
            RenameTableStatement {
                tables: table_pairs,
            },
        ))
    }
}

impl fmt::Display for RenameTableStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RENAME TABLE ")?;
        let table_name = self
            .tables
            .iter()
            .map(|(x, y)| {
                let old = match &x.schema {
                    Some(schema) => format!("{}.{}", schema, x.name),
                    None => x.name.clone(),
                };
                let new = match &y.schema {
                    Some(schema) => format!("{}.{}", schema, y.name),
                    None => y.name.clone(),
                };
                format!("{} TO {}", old, new)
            })
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}", table_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::table::Table;
    use dds::rename_table::RenameTableStatement;

    #[test]
    fn parse_drop_table() {
        let sqls = [
            "RENAME TABLE tbl_name1 TO tbl_name2;",
            "RENAME TABLE db1.tbl_name1 TO db2.tbl_name2;",
            "RENAME TABLE tbl_name1 TO tbl_name2, tbl_name3 TO tbl_name4;",
            "RENAME TABLE db1.tbl_name1 TO db2.tbl_name2, tbl_name3 TO tbl_name4;",
            "RENAME TABLE tbl_name1 TO tbl_name2, db3.tbl_name3 TO db4.tbl_name4;",
            "RENAME TABLE db1.tbl_name1 TO db2.tbl_name2, db3.tbl_name3 TO db4.tbl_name4;",
        ];

        let one_table = vec![(
            Table {
                name: String::from("tbl_name1"),
                alias: None,
                schema: None,
            },
            Table {
                name: String::from("tbl_name2"),
                alias: None,
                schema: None,
            },
        )];

        let one_table_with_schema = vec![(
            Table {
                name: String::from("tbl_name1"),
                alias: None,
                schema: Some(String::from("db1")),
            },
            Table {
                name: String::from("tbl_name2"),
                alias: None,
                schema: Some(String::from("db2")),
            },
        )];

        let two_tables = vec![
            (
                Table {
                    name: String::from("tbl_name1"),
                    alias: None,
                    schema: None,
                },
                Table {
                    name: String::from("tbl_name2"),
                    alias: None,
                    schema: None,
                },
            ),
            (
                Table {
                    name: String::from("tbl_name3"),
                    alias: None,
                    schema: None,
                },
                Table {
                    name: String::from("tbl_name4"),
                    alias: None,
                    schema: None,
                },
            ),
        ];

        let two_tables_with_schema = vec![
            (
                Table {
                    name: String::from("tbl_name1"),
                    alias: None,
                    schema: Some(String::from("db1")),
                },
                Table {
                    name: String::from("tbl_name2"),
                    alias: None,
                    schema: Some(String::from("db2")),
                },
            ),
            (
                Table {
                    name: String::from("tbl_name3"),
                    alias: None,
                    schema: Some(String::from("db3")),
                },
                Table {
                    name: String::from("tbl_name4"),
                    alias: None,
                    schema: Some(String::from("db4")),
                },
            ),
        ];

        let good_statements = [
            RenameTableStatement {
                tables: one_table.clone(),
            },
            RenameTableStatement {
                tables: one_table_with_schema.clone(),
            },
            RenameTableStatement {
                tables: two_tables.clone(),
            },
            RenameTableStatement {
                tables: vec![
                    (
                        Table {
                            name: String::from("tbl_name1"),
                            alias: None,
                            schema: Some(String::from("db1")),
                        },
                        Table {
                            name: String::from("tbl_name2"),
                            alias: None,
                            schema: Some(String::from("db2")),
                        },
                    ),
                    (
                        Table {
                            name: String::from("tbl_name3"),
                            alias: None,
                            schema: None,
                        },
                        Table {
                            name: String::from("tbl_name4"),
                            alias: None,
                            schema: None,
                        },
                    ),
                ],
            },
            RenameTableStatement {
                tables: vec![
                    (
                        Table {
                            name: String::from("tbl_name1"),
                            alias: None,
                            schema: None,
                        },
                        Table {
                            name: String::from("tbl_name2"),
                            alias: None,
                            schema: None,
                        },
                    ),
                    (
                        Table {
                            name: String::from("tbl_name3"),
                            alias: None,
                            schema: Some(String::from("db3")),
                        },
                        Table {
                            name: String::from("tbl_name4"),
                            alias: None,
                            schema: Some(String::from("db4")),
                        },
                    ),
                ],
            },
            RenameTableStatement {
                tables: two_tables_with_schema.clone(),
            },
        ];

        for i in 0..sqls.len() {
            let res = RenameTableStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, good_statements[i]);
        }
    }
}

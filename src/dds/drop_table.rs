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

use base::error::ParseSQLError;
use base::table::Table;
use base::CommonParser;

/// parse `DROP [TEMPORARY] TABLE [IF EXISTS]
///     tbl_name [, tbl_name] ...
///     [RESTRICT | CASCADE]`
#[derive(Default, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropTableStatement {
    pub if_temporary: bool,
    pub if_exists: bool,
    /// A name of a table, view, custom type, etc., possibly multipart, i.e. db.schema.obj
    pub tables: Vec<Table>,
    pub if_restrict: bool,
    pub if_cascade: bool,
}

impl DropTableStatement {
    pub fn parse(i: &str) -> IResult<&str, DropTableStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP "),
            opt(delimited(
                multispace0,
                tag_no_case("TEMPORARY"),
                multispace0,
            )),
            multispace0,
            tag_no_case("TABLE "),
            CommonParser::parse_if_exists,
            multispace0,
            many0(terminated(
                Table::without_alias,
                opt(CommonParser::ws_sep_comma),
            )),
            opt(delimited(multispace1, tag_no_case("RESTRICT"), multispace0)),
            opt(delimited(multispace1, tag_no_case("CASCADE"), multispace0)),
            CommonParser::statement_terminator,
        ));
        let (
            remaining_input,
            (
                _,
                opt_if_temporary,
                _,
                _,
                opt_if_exists,
                _,
                tables,
                opt_if_restrict,
                opt_if_cascade,
                _,
            ),
        ) = parser(i)?;

        Ok((
            remaining_input,
            DropTableStatement {
                if_temporary: opt_if_temporary.is_some(),
                tables,
                if_exists: opt_if_exists.is_some(),
                if_restrict: opt_if_restrict.is_some(),
                if_cascade: opt_if_cascade.is_some(),
            },
        ))
    }
}

impl fmt::Display for DropTableStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP")?;
        if self.if_temporary {
            write!(f, "TEMPORARY ")?;
        }
        write!(f, " TABLE")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }

        let table_name = self
            .tables
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, " {}", table_name)?;

        if self.if_restrict {
            write!(f, " RESTRICT")?;
        }
        if self.if_cascade {
            write!(f, " CASCADE")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::table::Table;
    use dds::drop_table::DropTableStatement;

    #[test]
    fn parse_drop_table() {
        let sqls = vec![
            "DROP  TABLE tbl_name;",
            "DROP TABLE  foo.tbl_name1, tbl_name2;",
            "DROP TEMPORARY  TABLE  bar.tbl_name",
            "DROP TEMPORARY TABLE tbl_name1, tbl_name2;",
            "DROP  TABLE  IF EXISTS  tbl_name;",
            "DROP TABLE IF EXISTS tbl_name1, foo.tbl_name2;",
            "DROP  TEMPORARY TABLE IF    EXISTS tbl_name;",
            "DROP TEMPORARY TABLE  IF EXISTS foo.tbl_name1, bar.tbl_name2;",
            "DROP  TABLE tbl_name RESTRICT",
            "DROP TABLE IF EXISTS tbl_name RESTRICT;",
            "DROP TEMPORARY TABLE tbl_name RESTRICT;",
            "DROP TEMPORARY  TABLE  IF  EXISTS tbl_name RESTRICT;",
            "DROP TABLE tbl_name1, tbl_name2 RESTRICT;",
            "DROP TABLE IF EXISTS tbl_name1, tbl_name2 RESTRICT;",
            "DROP TEMPORARY TABLE tbl_name1, tbl_name2 RESTRICT",
            "DROP TEMPORARY TABLE IF EXISTS tbl_name1, tbl_name2 RESTRICT;",
            "DROP TABLE tbl_name CASCADE",
            "DROP TABLE IF EXISTS tbl_name CASCADE;",
            "DROP TEMPORARY TABLE tbl_name CASCADE",
            "DROP TEMPORARY TABLE IF EXISTS tbl_name CASCADE;",
            "DROP TABLE tbl_name1, tbl_name2 CASCADE;",
            "DROP TABLE IF EXISTS tbl_name1, tbl_name2 CASCADE;",
            "DROP TEMPORARY TABLE tbl_name1, tbl_name2 CASCADE",
            "DROP TEMPORARY TABLE IF EXISTS tbl_name1, tbl_name2 CASCADE;",
        ];

        let one_table = vec![Table::from("tbl_name")];
        let two_tables = vec![Table::from("tbl_name1"), Table::from("tbl_name2")];

        let exp_statements = vec![
            DropTableStatement {
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                tables: vec![Table::from(("foo", "tbl_name1")), Table::from("tbl_name2")],
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                tables: vec![Table::from(("bar", "tbl_name"))],
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_exists: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_exists: true,
                tables: vec![Table::from("tbl_name1"), Table::from(("foo", "tbl_name2"))],
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_exists: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_exists: true,
                tables: vec![
                    Table::from(("foo", "tbl_name1")),
                    Table::from(("bar", "tbl_name2")),
                ],
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_restrict: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_exists: true,
                if_restrict: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_restrict: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_exists: true,
                if_restrict: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_restrict: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_exists: true,
                if_restrict: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_restrict: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_exists: true,
                if_restrict: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_cascade: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_exists: true,
                if_cascade: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_cascade: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_exists: true,
                if_cascade: true,
                tables: one_table.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_cascade: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_exists: true,
                if_cascade: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_cascade: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
            DropTableStatement {
                if_temporary: true,
                if_exists: true,
                if_cascade: true,
                tables: two_tables.clone(),
                ..DropTableStatement::default()
            },
        ];

        for i in 0..sqls.len() {
            let res = DropTableStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i]);
        }
    }
}

use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::table::Table;
use common::statement_terminator;

/// TRUNCATE [TABLE] tbl_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TruncateTableStatement {
    pub table: Table,
}

impl TruncateTableStatement {
    /// TRUNCATE [TABLE] tbl_name
    pub fn parse(i: &str) -> IResult<&str, TruncateTableStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("TRUNCATE "),
            multispace0,
            opt(tag_no_case("TABLE ")),
            multispace0,
            Table::without_alias,
            statement_terminator,
        ));
        let (remaining_input, (_, _, _, _, table, _)) = parser(i)?;

        Ok((remaining_input, TruncateTableStatement { table }))
    }
}

impl fmt::Display for TruncateTableStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "TRUNCATE TABLE ")?;
        if self.table.schema.is_some() {
            write!(f, "{}.", self.table.schema.clone().unwrap())?;
        }
        write!(f, " {}", self.table.name.clone())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::table::Table;
    use dds::truncate_table::TruncateTableStatement;

    #[test]
    fn test_parse_truncate_table() {
        let good_sqls = vec![
            "TRUNCATE table_name",
            "TRUNCATE     db_name.table_name",
            "TRUNCATE   TABLE db_name.table_name",
            "TRUNCATE TABLE table_name",
        ];

        let table_name = Table::from("table_name");
        let table_name_with_schema = Table::from(("db_name", "table_name"));

        let good_statements = vec![
            TruncateTableStatement {
                table: table_name.clone(),
            },
            TruncateTableStatement {
                table: table_name_with_schema.clone(),
            },
            TruncateTableStatement {
                table: table_name_with_schema.clone(),
            },
            TruncateTableStatement {
                table: table_name.clone(),
            },
        ];

        for i in 0..good_sqls.len() {
            assert_eq!(
                TruncateTableStatement::parse(good_sqls[i]).unwrap().1,
                good_statements[i]
            );
        }

        let bad_sqls = vec![
            "TRUNCATE table_name as abc",
            "TRUNCATE table_name abc",
            "TRUNCATE SCHEMA table_name1, table_name2;",
            "TRUNCATE TABLE IF EXISTS table_name;",
            "DROP DATABASE IFEXISTS db_name;",
        ];

        for i in 0..bad_sqls.len() {
            assert!(TruncateTableStatement::parse(bad_sqls[i]).is_err())
        }
    }
}

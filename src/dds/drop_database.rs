use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// DROP {DATABASE | SCHEMA} [IF EXISTS] db_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropDatabaseStatement {
    pub if_exists: bool,
    pub name: String,
}

impl DropDatabaseStatement {
    pub fn parse(i: &str) -> IResult<&str, DropDatabaseStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP "),
            multispace0,
            alt((tag_no_case("DATABASE "), tag_no_case("SCHEMA "))),
            CommonParser::parse_if_exists,
            multispace0,
            CommonParser::sql_identifier,
            CommonParser::statement_terminator,
        ));
        let (remaining_input, (_, _, _, opt_if_exists, _, database, _)) = parser(i)?;

        let name = String::from(database);

        Ok((
            remaining_input,
            DropDatabaseStatement {
                name,
                if_exists: opt_if_exists.is_some(),
            },
        ))
    }
}

impl fmt::Display for DropDatabaseStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP DATABASE ")?;
        if self.if_exists {
            write!(f, "IF EXISTS ")?;
        }
        let database = self.name.clone();
        write!(f, " {}", database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_database::DropDatabaseStatement;

    #[test]
    fn test_parse_drop_database() {
        let good_sqls = [
            "DROP DATABASE db_name",
            "DROP SCHEMA db_name;",
            "DROP DATABASE IF EXISTS db_name;",
            "DROP DATABASE IF  EXISTS db_name;",
            "DROP SCHEMA IF EXISTS db_name",
            "DROP SCHEMA IF      EXISTS db_name",
        ];

        let database_name = String::from("db_name");

        let good_statements = [
            DropDatabaseStatement {
                if_exists: false,
                name: database_name.clone(),
            },
            DropDatabaseStatement {
                if_exists: false,
                name: database_name.clone(),
            },
            DropDatabaseStatement {
                if_exists: true,
                name: database_name.clone(),
            },
            DropDatabaseStatement {
                if_exists: true,
                name: database_name.clone(),
            },
            DropDatabaseStatement {
                if_exists: true,
                name: database_name.clone(),
            },
            DropDatabaseStatement {
                if_exists: true,
                name: database_name.clone(),
            },
        ];

        for i in 0..good_sqls.len() {
            assert_eq!(
                DropDatabaseStatement::parse(good_sqls[i]).unwrap().1,
                good_statements[i]
            );
        }

        let bad_sqls = [
            "DROP DATABASE db_name_1, db_name2",
            "DROP SCHEMA db_name_1, db_name2;",
            "DROP DATABASE IF NOT EXISTS db_name;",
            "DROP DATABASE IFEXISTS db_name;",
            "DROP SCHEMA IF EXISTS db_name_1, db_name_2",
            "DROP SCHEMA IF      EXISTS db_name_1, db_name_2",
            "DROP TABLE IF EXISTS db_name_1",
            "DROP DATABASE2",
        ];

        for i in 0..bad_sqls.len() {
            println!("{} / {}", i + 1, bad_sqls.len());
            assert!(DropDatabaseStatement::parse(bad_sqls[i]).is_err())
        }
    }
}

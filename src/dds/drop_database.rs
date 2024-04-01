use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::error::VerboseError;
use nom::sequence::tuple;
use nom::IResult;

use common::{parse_if_exists, sql_identifier, statement_terminator};

/// DROP {DATABASE | SCHEMA} [IF EXISTS] db_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropDatabaseStatement {
    pub if_exists: bool,
    pub name: String,
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

pub fn drop_database(i: &str) -> IResult<&str, DropDatabaseStatement, VerboseError<&str>> {
    let mut parser = tuple((
        tag_no_case("DROP "),
        multispace0,
        alt((tag_no_case("DATABASE "), tag_no_case("SCHEMA "))),
        parse_if_exists,
        multispace0,
        sql_identifier,
        statement_terminator,
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

#[cfg(test)]
mod tests {
    use dds::drop_database::{drop_database, DropDatabaseStatement};

    #[test]
    fn test_parse_drop_database() {
        let good_sqls = vec![
            "DROP DATABASE db_name",
            "DROP SCHEMA db_name;",
            "DROP DATABASE IF EXISTS db_name;",
            "DROP DATABASE IF  EXISTS db_name;",
            "DROP SCHEMA IF EXISTS db_name",
            "DROP SCHEMA IF      EXISTS db_name",
        ];

        let database_name = String::from("db_name");

        let good_statements = vec![
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
            assert_eq!(drop_database(good_sqls[i]).unwrap().1, good_statements[i]);
        }

        let bad_sqls = vec![
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
            assert!(drop_database(bad_sqls[i]).is_err())
        }
    }
}
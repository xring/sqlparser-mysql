use core::fmt;
use std::fmt::Formatter;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::{CommonParser, DefaultOrZeroOrOne};

/// parse `ALTER {DATABASE | SCHEMA} [db_name]
///     alter_option ...`
///
/// `alter_option: {
///     [DEFAULT] CHARACTER SET [=] charset_name
///   | [DEFAULT] COLLATE [=] collation_name
///   | [DEFAULT] ENCRYPTION [=] {'Y' | 'N'}
///   | READ ONLY [=] {DEFAULT | 0 | 1}
/// }`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AlterDatabaseStatement {
    // we parse SQL, db_name is needed
    pub db_name: String,
    pub alter_options: Vec<AlterDatabaseOption>,
}

impl AlterDatabaseStatement {
    pub fn parse(i: &str) -> IResult<&str, AlterDatabaseStatement, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ALTER"),
                multispace0,
                alt((tag_no_case("DATABASE"), tag_no_case("SCHEMA"))),
                multispace1,
                map(CommonParser::sql_identifier, String::from),
                multispace1,
                many1(terminated(AlterDatabaseOption::parse, multispace0)),
            )),
            |x| AlterDatabaseStatement {
                db_name: x.4,
                alter_options: x.6,
            },
        )(i)
    }
}

impl fmt::Display for AlterDatabaseStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ALTER DATABASE")?;
        let database = self.db_name.clone();
        write!(f, " {}", database)?;
        for alter_option in self.alter_options.iter() {
            write!(f, " {}", alter_option)?;
        }
        Ok(())
    }
}

/// `alter_option: {
///     [DEFAULT] CHARACTER SET [=] charset_name
///   | [DEFAULT] COLLATE [=] collation_name
///   | [DEFAULT] ENCRYPTION [=] {'Y' | 'N'}
///   | READ ONLY [=] {DEFAULT | 0 | 1}
/// }`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlterDatabaseOption {
    CharacterSet(String),
    Collate(String),
    Encryption(bool),
    ReadOnly(DefaultOrZeroOrOne),
}

impl AlterDatabaseOption {
    fn parse(i: &str) -> IResult<&str, AlterDatabaseOption, ParseSQLError<&str>> {
        // [DEFAULT] CHARACTER SET [=] charset_name
        let character = map(
            tuple((
                opt(tag_no_case("DEFAULT")),
                multispace1,
                tuple((
                    tag_no_case("CHARACTER"),
                    multispace1,
                    tag_no_case("SET"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                )),
                map(CommonParser::sql_identifier, String::from),
                multispace0,
            )),
            |(_, _, _, charset_name, _)| AlterDatabaseOption::CharacterSet(charset_name),
        );

        // [DEFAULT] COLLATE [=] collation_name
        let collate = map(
            tuple((
                opt(tag_no_case("DEFAULT")),
                multispace1,
                map(
                    tuple((
                        tag_no_case("COLLATE"),
                        multispace0,
                        opt(tag("=")),
                        multispace0,
                        CommonParser::sql_identifier,
                        multispace0,
                    )),
                    |(_, _, _, _, collation_name, _)| String::from(collation_name),
                ),
                multispace0,
            )),
            |(_, _, collation_name, _)| AlterDatabaseOption::Collate(collation_name),
        );

        // [DEFAULT] ENCRYPTION [=] {'Y' | 'N'}
        let encryption = map(
            tuple((
                opt(tag_no_case("DEFAULT")),
                multispace1,
                tag_no_case("ENCRYPTION"),
                multispace1,
                opt(tag("=")),
                multispace0,
                alt((map(tag("'Y'"), |_| true), map(tag("'N'"), |_| false))),
                multispace0,
            )),
            |x| AlterDatabaseOption::Encryption(x.6),
        );

        // READ ONLY [=] {DEFAULT | 0 | 1}
        let read_only = map(
            tuple((
                opt(tag_no_case("READ")),
                multispace1,
                tag_no_case("ONLY "),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
            )),
            |x| AlterDatabaseOption::ReadOnly(x.6),
        );

        alt((character, collate, encryption, read_only))(i)
    }
}

impl fmt::Display for AlterDatabaseOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlterDatabaseOption::CharacterSet(str) => write!(f, " CHARACTER SET {}", str)?,
            AlterDatabaseOption::Collate(str) => write!(f, " COLLATE {}", str)?,
            AlterDatabaseOption::Encryption(bl) => {
                if *bl {
                    write!(f, " ENCRYPTION 'Y'",)?
                } else {
                    write!(f, " ENCRYPTION 'N'",)?
                }
            }
            AlterDatabaseOption::ReadOnly(val) => write!(f, " READ ONLY {}", val)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::DefaultOrZeroOrOne;
    use dds::alter_database::{AlterDatabaseOption, AlterDatabaseStatement};

    #[test]
    fn test_alter_database() {
        let sqls = ["ALTER DATABASE test_db DEFAULT CHARACTER SET = utf8mb4 \
            DEFAULT COLLATE utf8mb4_unicode_ci DEFAULT ENCRYPTION = 'Y' READ ONLY = 1;"];
        let exp_statements = [AlterDatabaseStatement {
            db_name: "test_db".to_string(),
            alter_options: vec![
                AlterDatabaseOption::CharacterSet("utf8mb4".to_string()),
                AlterDatabaseOption::Collate("utf8mb4_unicode_ci".to_string()),
                AlterDatabaseOption::Encryption(true),
                AlterDatabaseOption::ReadOnly(DefaultOrZeroOrOne::One),
            ],
        }];
        for i in 0..sqls.len() {
            let res = AlterDatabaseStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i]);
        }
    }
}

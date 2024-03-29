use core::fmt;
use std::fmt::Formatter;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{terminated, tuple};
use nom::IResult;

use common_parsers::sql_identifier;
use common_statement::DefaultOrZeroOrOne;

/// ALTER {DATABASE | SCHEMA} [db_name]
///     alter_option ...
///
/// alter_option: {
///     [DEFAULT] CHARACTER SET [=] charset_name
///   | [DEFAULT] COLLATE [=] collation_name
///   | [DEFAULT] ENCRYPTION [=] {'Y' | 'N'}
///   | READ ONLY [=] {DEFAULT | 0 | 1}
/// }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AlterDatabaseStatement {
    // we parse SQL, db_name is needed
    pub name: String,
    pub alter_options: Vec<AlterDatabaseOption>,
}

impl fmt::Display for AlterDatabaseStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ALTER DATABASE")?;
        let database = self.name.clone();
        write!(f, " {}", database)?;
        for alter_option in self.alter_options.iter() {
            write!(f, " {}", alter_option)?;
        }
        Ok(())
    }
}

pub fn alter_database_parser(i: &str) -> IResult<&str, AlterDatabaseStatement> {
    map(
        tuple((
            tag_no_case("ALTER "),
            multispace0,
            alt((tag_no_case("DATABASE "), tag_no_case("SCHEMA "))),
            multispace0,
            map(sql_identifier, |x| String::from(x)),
            multispace1,
            many1(terminated(alter_database_option, multispace0)),
        )),
        |x| AlterDatabaseStatement {
            name: x.4,
            alter_options: x.6,
        },
    )(i)
}

/// alter_option: {
///     [DEFAULT] CHARACTER SET [=] charset_name
///   | [DEFAULT] COLLATE [=] collation_name
///   | [DEFAULT] ENCRYPTION [=] {'Y' | 'N'}
///   | READ ONLY [=] {DEFAULT | 0 | 1}
/// }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlterDatabaseOption {
    CharacterSet(String),
    Collate(String),
    Encryption(bool),
    ReadOnly(DefaultOrZeroOrOne),
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

fn alter_database_option(i: &str) -> IResult<&str, AlterDatabaseOption> {
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
            map(sql_identifier, |x| String::from(x)),
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
                    sql_identifier,
                    multispace0,
                )),
                |(_, _, _, _, collation_name, _)| {
                    String::from(collation_name)
                },
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
            alt((
                map(tag_no_case("0"), |_| DefaultOrZeroOrOne::Zero),
                map(tag_no_case("1"), |_| DefaultOrZeroOrOne::One),
                map(tag_no_case("DEFAULT"), |_| DefaultOrZeroOrOne::Default),
            )),
        )),
        |x| AlterDatabaseOption::ReadOnly(x.6),
    );

    alt((character, collate, encryption, read_only))(i)
}

#[cfg(test)]
mod test {
    use data_definition_statement::alter_database::alter_database_parser;

    #[test]
    fn test_alter_database() {
        let sqls = vec![
            "ALTER DATABASE test_db DEFAULT CHARACTER SET utf8mb4 DEFAULT COLLATE = utf8mb4_unicode_ci;",
            "ALTER DATABASE test_db DEFAULT CHARACTER SET = utf8mb4 DEFAULT COLLATE utf8mb4_unicode_ci DEFAULT ENCRYPTION = 'Y' READ ONLY = 1;",
            "ALTER DATABASE test_db DEFAULT CHARACTER SET utf8mb4",
            "ALTER DATABASE test_db DEFAULT COLLATE = utf8mb4_unicode_ci;",
            "ALTER DATABASE test_db DEFAULT ENCRYPTION = 'Y';",
            "ALTER DATABASE test_db READ ONLY = 1;",
        ];
        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = alter_database_parser(sqls[i]);
            assert!(res.is_ok());
        }
    }
}

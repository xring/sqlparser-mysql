use std::fmt;
use std::io::BufRead;
use std::str;

use nom::branch::alt;
use nom::combinator::map;
use nom::{IResult, Offset};

use base::error::ParseSQLError;
use das::set_statement::SetStatement;
use dds::alter_database::AlterDatabaseStatement;
use dds::alter_table::AlterTableStatement;
use dds::create_index::CreateIndexStatement;
use dds::create_table::CreateTableStatement;
use dds::drop_database::DropDatabaseStatement;
use dds::drop_event::DropEventStatement;
use dds::drop_function::DropFunctionStatement;
use dds::drop_index::DropIndexStatement;
use dds::drop_logfile_group::DropLogfileGroupStatement;
use dds::drop_procedure::DropProcedureStatement;
use dds::drop_server::DropServerStatement;
use dds::drop_spatial_reference_system::DropSpatialReferenceSystemStatement;
use dds::drop_table::DropTableStatement;
use dds::drop_tablespace::DropTablespaceStatement;
use dds::drop_trigger::DropTriggerStatement;
use dds::drop_view::DropViewStatement;
use dds::rename_table::RenameTableStatement;
use dds::truncate_table::TruncateTableStatement;
use dms::compound_select::CompoundSelectStatement;
use dms::delete::DeleteStatement;
use dms::insert::InsertStatement;
use dms::select::SelectStatement;
use dms::update::UpdateStatement;

pub struct Parser;

impl Parser {
    pub fn parse(config: &ParseConfig, input: &str) -> Result<Statement, String> {
        let input = input.trim();

        let dds_parser = alt((
            map(AlterDatabaseStatement::parse, |parsed| {
                Statement::AlterDatabase(parsed)
            }),
            map(AlterTableStatement::parse, |parsed| {
                Statement::AlterTable(parsed)
            }),
            map(CreateIndexStatement::parse, |parsed| {
                Statement::CreateIndex(parsed)
            }),
            map(CreateTableStatement::parse, |parsed| {
                Statement::CreateTable(parsed)
            }),
            map(DropDatabaseStatement::parse, |parsed| {
                Statement::DropDatabase(parsed)
            }),
            map(DropEventStatement::parse, |parsed| {
                Statement::DropEvent(parsed)
            }),
            map(DropFunctionStatement::parse, |parsed| {
                Statement::DropFunction(parsed)
            }),
            map(DropIndexStatement::parse, |parsed| {
                Statement::DropIndex(parsed)
            }),
            map(DropLogfileGroupStatement::parse, |parsed| {
                Statement::DropLogfileGroup(parsed)
            }),
            map(DropProcedureStatement::parse, |parsed| {
                Statement::DropProcedure(parsed)
            }),
            map(DropServerStatement::parse, |parsed| {
                Statement::DropServer(parsed)
            }),
            map(DropSpatialReferenceSystemStatement::parse, |parsed| {
                Statement::DropSpatialReferenceSystem(parsed)
            }),
            map(DropTableStatement::parse, |parsed| {
                Statement::DropTable(parsed)
            }),
            map(DropTablespaceStatement::parse, |parsed| {
                Statement::DropTableSpace(parsed)
            }),
            map(DropTriggerStatement::parse, |parsed| {
                Statement::DropTrigger(parsed)
            }),
            map(DropViewStatement::parse, |parsed| {
                Statement::DropView(parsed)
            }),
            map(RenameTableStatement::parse, |parsed| {
                Statement::RenameTable(parsed)
            }),
            map(TruncateTableStatement::parse, |parsed| {
                Statement::TruncateTable(parsed)
            }),
        ));

        let das_parser = alt((map(SetStatement::parse, |parsed| Statement::Set(parsed)),));

        let dms_parser = alt((
            map(SelectStatement::parse, |parsed| Statement::Select(parsed)),
            map(CompoundSelectStatement::parse, |parsed| {
                Statement::CompoundSelect(parsed)
            }),
            map(InsertStatement::parse, |parsed| Statement::Insert(parsed)),
            map(DeleteStatement::parse, |parsed| Statement::Delete(parsed)),
            map(UpdateStatement::parse, |parsed| Statement::Update(parsed)),
        ));

        let mut parser = alt((dds_parser, dms_parser, das_parser));

        return match parser(input) {
            Ok(result) => Ok(result.1),
            Err(nom::Err::Error(err)) => {
                if config.log_with_backtrace {
                    println!(">>>>>>>>>>>>>>>>>>>>");
                    for error in &err.errors {
                        println!("{:?} :: {:?}", error.0, error.1)
                    }
                    println!("<<<<<<<<<<<<<<<<<<<<");
                }

                let msg = err.errors[0].0;
                let err_msg = format!("failed to parse sql, error near `{}`", msg);
                Err(err_msg)
            }
            _ => Err(String::from("failed to parse sql: other error")),
        };
    }
}

pub struct ParseConfig {
    pub log_with_backtrace: bool,
}

impl Default for ParseConfig {
    fn default() -> Self {
        ParseConfig {
            log_with_backtrace: false,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // DDS
    AlterDatabase(AlterDatabaseStatement),
    AlterTable(AlterTableStatement),
    CreateIndex(CreateIndexStatement),
    CreateTable(CreateTableStatement),
    DropDatabase(DropDatabaseStatement),
    DropEvent(DropEventStatement),
    DropFunction(DropFunctionStatement),
    DropIndex(DropIndexStatement),
    DropLogfileGroup(DropLogfileGroupStatement),
    DropProcedure(DropProcedureStatement),
    DropServer(DropServerStatement),
    DropSpatialReferenceSystem(DropSpatialReferenceSystemStatement),
    DropTable(DropTableStatement),
    DropTableSpace(DropTablespaceStatement),
    DropTrigger(DropTriggerStatement),
    DropView(DropViewStatement),
    RenameTable(RenameTableStatement),
    TruncateTable(TruncateTableStatement),
    // DAS
    Set(SetStatement),
    // HISTORY
    Insert(InsertStatement),
    CompoundSelect(CompoundSelectStatement),
    Select(SelectStatement),
    Delete(DeleteStatement),
    Update(UpdateStatement),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Statement::Select(ref select) => write!(f, "{}", select),
            Statement::Insert(ref insert) => write!(f, "{}", insert),
            Statement::CreateTable(ref create) => write!(f, "{}", create),
            Statement::Delete(ref delete) => write!(f, "{}", delete),
            Statement::DropTable(ref drop) => write!(f, "{}", drop),
            Statement::DropDatabase(ref drop) => write!(f, "{}", drop),
            Statement::TruncateTable(ref drop) => write!(f, "{}", drop),
            Statement::Update(ref update) => write!(f, "{}", update),
            Statement::Set(ref set) => write!(f, "{}", set),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use base::table::Table;
    use dms::insert::InsertStatement;

    use super::*;

    #[test]
    fn hash_query() {
        let str = "INSERT INTO users VALUES (42, \"test\");";
        let config = ParseConfig::default();
        let res = Parser::parse(&config, str);
        assert!(res.is_ok());

        let expected = Statement::Insert(InsertStatement {
            table: Table::from("users"),
            fields: None,
            data: vec![vec![42.into(), "test".into()]],
            ..Default::default()
        });
        let mut h0 = DefaultHasher::new();
        let mut h1 = DefaultHasher::new();
        res.unwrap().hash(&mut h0);
        expected.hash(&mut h1);
        assert_eq!(h0.finish(), h1.finish());
    }

    #[test]
    fn parse_byte_slice() {
        let str = "INSERT INTO users VALUES (42, \"test\");";
        let config = ParseConfig::default();
        let res = Parser::parse(&config, str);
        assert!(res.is_ok());
    }

    #[test]
    fn parse_byte_vector() {
        let str = "INSERT INTO users VALUES (42, \"test\");";
        let config = ParseConfig::default();
        let res = Parser::parse(&config, str);
        assert!(res.is_ok());
    }

    #[test]
    fn display_select_query() {
        let str0 = "SELECT * FROM users";
        let str1 = "SELECT * FROM users AS u";
        let str2 = "SELECT name, password FROM users AS u";
        let str3 = "SELECT name, password FROM users AS u WHERE user_id = '1'";
        let str4 = "SELECT name, password FROM users AS u WHERE user = 'aaa' AND password = 'xxx'";
        let str5 = "SELECT name * 2 AS double_name FROM users";
        let config = ParseConfig::default();

        let res0 = Parser::parse(&config, str0);
        let res1 = Parser::parse(&config, str1);
        let res2 = Parser::parse(&config, str2);
        let res3 = Parser::parse(&config, str3);
        let res4 = Parser::parse(&config, str4);
        let res5 = Parser::parse(&config, str5);

        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert!(res2.is_ok());
        assert!(res3.is_ok());
        assert!(res4.is_ok());
        assert!(res5.is_ok());

        assert_eq!(str0, format!("{}", res0.unwrap()));
        assert_eq!(str1, format!("{}", res1.unwrap()));
        assert_eq!(str2, format!("{}", res2.unwrap()));
        assert_eq!(str3, format!("{}", res3.unwrap()));
        assert_eq!(str4, format!("{}", res4.unwrap()));
        assert_eq!(str5, format!("{}", res5.unwrap()));
    }

    #[test]
    fn format_select_query() {
        let str1 = "select * from users u";
        let str2 = "select name,password from users u;";
        let str3 = "select name,password from users u WHERE user_id='1'";

        let expected1 = "SELECT * FROM users AS u";
        let expected2 = "SELECT name, password FROM users AS u";
        let expected3 = "SELECT name, password FROM users AS u WHERE user_id = '1'";
        let config = ParseConfig::default();

        let res1 = Parser::parse(&config, str1);
        let res2 = Parser::parse(&config, str2);
        let res3 = Parser::parse(&config, str3);

        assert!(res1.is_ok());
        assert!(res2.is_ok());
        assert!(res3.is_ok());

        assert_eq!(expected1, format!("{}", res1.unwrap()));
        assert_eq!(expected2, format!("{}", res2.unwrap()));
        assert_eq!(expected3, format!("{}", res3.unwrap()));
    }

    #[test]
    fn format_select_query_with_where_clause() {
        let str0 = "select name, password from users as u where user='aaa' and password= 'xxx'";
        let str1 = "select name, password from users as u where user=? and password =?";

        let expected0 =
            "SELECT name, password FROM users AS u WHERE user = 'aaa' AND password = 'xxx'";
        let expected1 = "SELECT name, password FROM users AS u WHERE user = ? AND password = ?";
        let config = ParseConfig::default();
        let res0 = Parser::parse(&config, str0);
        let res1 = Parser::parse(&config, str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn format_select_query_with_function() {
        let str1 = "select count(*) from users";
        let expected1 = "SELECT count(*) FROM users";
        let config = ParseConfig::default();
        let res1 = Parser::parse(&config, str1);
        assert!(res1.is_ok());
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn display_insert_query() {
        let str = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
        let config = ParseConfig::default();
        let res = Parser::parse(&config, str);
        assert!(res.is_ok());
        assert_eq!(str, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_update_query() {
        let str = "update users set name=42, password='xxx' where id=1";
        let expected = "UPDATE users SET name = 42, password = 'xxx' WHERE id = 1";
        let config = ParseConfig::default();
        let res = Parser::parse(&config, str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_delete_query_with_where_clause() {
        let str0 = "delete from users where user='aaa' and password= 'xxx'";
        let str1 = "delete from users where user=? and password =?";

        let expected0 = "DELETE FROM users WHERE user = 'aaa' AND password = 'xxx'";
        let expected1 = "DELETE FROM users WHERE user = ? AND password = ?";
        let config = ParseConfig::default();
        let res0 = Parser::parse(&config, str0);
        let res1 = Parser::parse(&config, str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn format_query_with_escaped_keyword() {
        let str0 = "delete from articles where `key`='aaa'";
        let str1 = "delete from `where` where user=?";

        let expected0 = "DELETE FROM articles WHERE `key` = 'aaa'";
        let expected1 = "DELETE FROM `where` WHERE user = ?";
        let config = ParseConfig::default();
        let res0 = Parser::parse(&config, str0);
        let res1 = Parser::parse(&config, str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }
}

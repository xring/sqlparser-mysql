use std::fmt;
use std::io::BufRead;
use std::str;

use das::SetStatement;
use dds::{
    AlterDatabaseStatement, AlterTableStatement, CreateIndexStatement, CreateTableStatement,
    DropDatabaseStatement, DropEventStatement, DropFunctionStatement, DropIndexStatement,
    DropLogfileGroupStatement, DropProcedureStatement, DropServerStatement,
    DropSpatialReferenceSystemStatement, DropTableStatement, DropTablespaceStatement,
    DropTriggerStatement, DropViewStatement, RenameTableStatement, TruncateTableStatement,
};
use dms::{
    CompoundSelectStatement, DeleteStatement, InsertStatement, SelectStatement, UpdateStatement,
};
use nom::branch::alt;
use nom::combinator::map;
use nom::Offset;

pub struct Parser;

impl Parser {
    pub fn parse(config: &ParseConfig, input: &str) -> Result<Statement, String> {
        let input = input.trim();

        let dds_parser = alt((
            map(AlterDatabaseStatement::parse, Statement::AlterDatabase),
            map(AlterTableStatement::parse, Statement::AlterTable),
            map(CreateIndexStatement::parse, Statement::CreateIndex),
            map(CreateTableStatement::parse, Statement::CreateTable),
            map(DropDatabaseStatement::parse, Statement::DropDatabase),
            map(DropEventStatement::parse, Statement::DropEvent),
            map(DropFunctionStatement::parse, Statement::DropFunction),
            map(DropIndexStatement::parse, Statement::DropIndex),
            map(
                DropLogfileGroupStatement::parse,
                Statement::DropLogfileGroup,
            ),
            map(DropProcedureStatement::parse, Statement::DropProcedure),
            map(DropServerStatement::parse, Statement::DropServer),
            map(
                DropSpatialReferenceSystemStatement::parse,
                Statement::DropSpatialReferenceSystem,
            ),
            map(DropTableStatement::parse, Statement::DropTable),
            map(DropTablespaceStatement::parse, Statement::DropTableSpace),
            map(DropTriggerStatement::parse, Statement::DropTrigger),
            map(DropViewStatement::parse, Statement::DropView),
            map(RenameTableStatement::parse, Statement::RenameTable),
            map(TruncateTableStatement::parse, Statement::TruncateTable),
        ));

        let das_parser = alt((map(SetStatement::parse, Statement::Set),));

        let dms_parser = alt((
            map(SelectStatement::parse, Statement::Select),
            map(CompoundSelectStatement::parse, Statement::CompoundSelect),
            map(InsertStatement::parse, Statement::Insert),
            map(DeleteStatement::parse, Statement::Delete),
            map(UpdateStatement::parse, Statement::Update),
        ));

        let mut parser = alt((dds_parser, dms_parser, das_parser));

        match parser(input) {
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
        }
    }
}

#[derive(Default)]
pub struct ParseConfig {
    pub log_with_backtrace: bool,
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

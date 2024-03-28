use std::fmt;
use std::str;

use data_definition_statement::alter_database::{alter_database_parser, AlterDatabaseStatement};
use data_definition_statement::alter_table::AlterTableStatement;
use data_definition_statement::create_table::CreateTableStatement;
use data_definition_statement::drop_database::{drop_database_parser, DropDatabaseStatement};
use data_definition_statement::drop_table::{drop_table_parser, DropTableStatement};
use data_definition_statement::rename_table::{rename_table_parser, RenameTableStatement};
use data_definition_statement::truncate_table::{truncate_table_parser, TruncateTableStatement};
use data_definition_statement::{alter_table_parser, create_table_parser};
use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;
use zz_compound_select::{compound_selection, CompoundSelectStatement};
use zz_create::{creation, view_creation, CreateViewStatement};
use zz_delete::{deletion, DeleteStatement};
use zz_insert::{insertion, InsertStatement};
use zz_select::{selection, SelectStatement};
use zz_set::{set, SetStatement};
use zz_update::{updating, UpdateStatement};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SQLStatement {
    AlterDatabase(AlterDatabaseStatement),
    AlterTable(AlterTableStatement),
    CreateTable(CreateTableStatement),
    DropDatabase(DropDatabaseStatement),
    DropTable(DropTableStatement),
    RenameTable(RenameTableStatement),
    TruncateTable(TruncateTableStatement),
    // HISTORY
    CreateView(CreateViewStatement),
    Insert(InsertStatement),
    CompoundSelect(CompoundSelectStatement),
    Select(SelectStatement),
    Delete(DeleteStatement),
    Update(UpdateStatement),
    Set(SetStatement),
}

impl fmt::Display for SQLStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SQLStatement::Select(ref select) => write!(f, "{}", select),
            SQLStatement::Insert(ref insert) => write!(f, "{}", insert),
            SQLStatement::CreateTable(ref create) => write!(f, "{}", create),
            SQLStatement::CreateView(ref create) => write!(f, "{}", create),
            SQLStatement::Delete(ref delete) => write!(f, "{}", delete),
            SQLStatement::DropTable(ref drop) => write!(f, "{}", drop),
            SQLStatement::DropDatabase(ref drop) => write!(f, "{}", drop),
            SQLStatement::TruncateTable(ref drop) => write!(f, "{}", drop),
            SQLStatement::Update(ref update) => write!(f, "{}", update),
            SQLStatement::Set(ref set) => write!(f, "{}", set),
            _ => unimplemented!(),
        }
    }
}

pub fn parse_sql(i: &[u8]) -> IResult<&[u8], SQLStatement> {
    alt((
        // ALTER DATABASE
        map(alter_database_parser, |ad| SQLStatement::AlterDatabase(ad)),
        // ALTER TABLE
        map(alter_table_parser, |at| SQLStatement::AlterTable(at)),
        // CREATE TABLE
        map(create_table_parser, |ct| SQLStatement::CreateTable(ct)),
        // DROP DATABASE
        map(drop_database_parser, |dt| SQLStatement::DropDatabase(dt)),
        // DROP TABLE
        map(drop_table_parser, |dt| SQLStatement::DropTable(dt)),
        // RENAME TABLE
        map(rename_table_parser, |dt| SQLStatement::RenameTable(dt)),
        // TRUNCATE TABLE
        map(truncate_table_parser, |dt| SQLStatement::TruncateTable(dt)),
        // HISTORY
        map(insertion, |i| SQLStatement::Insert(i)),
        map(compound_selection, |cs| SQLStatement::CompoundSelect(cs)),
        map(selection, |s| SQLStatement::Select(s)),
        map(deletion, |d| SQLStatement::Delete(d)),
        map(updating, |u| SQLStatement::Update(u)),
        map(set, |s| SQLStatement::Set(s)),
        map(view_creation, |vc| SQLStatement::CreateView(vc)),
    ))(i)
}

pub fn parse_query_bytes<T>(input: T) -> Result<SQLStatement, &'static str>
where
    T: AsRef<[u8]>,
{
    match parse_sql(input.as_ref()) {
        Ok((_, o)) => Ok(o),
        Err(err) => {
            println!("{:?}", err);
            Err("failed to parse query")
        }
    }
}

pub fn parse_query<T>(input: T) -> Result<SQLStatement, &'static str>
where
    T: AsRef<str>,
{
    parse_query_bytes(input.as_ref().trim().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::table::Table;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn hash_query() {
        let str = "INSERT INTO users VALUES (42, \"test\");";
        let res = parse_query(str);
        assert!(res.is_ok());

        let expected = SQLStatement::Insert(InsertStatement {
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
    fn trim_query() {
        let str = "   INSERT INTO users VALUES (42, \"test\");     ";
        let res = parse_query(str);
        assert!(res.is_ok());
    }

    #[test]
    fn parse_byte_slice() {
        let str: &[u8] = b"INSERT INTO users VALUES (42, \"test\");";
        let res = parse_query_bytes(str);
        assert!(res.is_ok());
    }

    #[test]
    fn parse_byte_vector() {
        let str: Vec<u8> = b"INSERT INTO users VALUES (42, \"test\");".to_vec();
        let res = parse_query_bytes(&str);
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

        let res0 = parse_query(str0);
        let res1 = parse_query(str1);
        let res2 = parse_query(str2);
        let res3 = parse_query(str3);
        let res4 = parse_query(str4);
        let res5 = parse_query(str5);

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

        let res1 = parse_query(str1);
        let res2 = parse_query(str2);
        let res3 = parse_query(str3);

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

        let res0 = parse_query(str0);
        let res1 = parse_query(str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn format_select_query_with_function() {
        let str1 = "select count(*) from users";
        let expected1 = "SELECT count(*) FROM users";

        let res1 = parse_query(str1);
        assert!(res1.is_ok());
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn display_insert_query() {
        let str = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
        let res = parse_query(str);
        assert!(res.is_ok());
        assert_eq!(str, format!("{}", res.unwrap()));
    }

    #[test]
    fn display_insert_query_no_columns() {
        let str = "INSERT INTO users VALUES ('aaa', 'xxx')";
        let expected = "INSERT INTO users VALUES ('aaa', 'xxx')";
        let res = parse_query(str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_insert_query() {
        let str = "insert into users (name, password) values ('aaa', 'xxx')";
        let expected = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
        let res = parse_query(str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_update_query() {
        let str = "update users set name=42, password='xxx' where id=1";
        let expected = "UPDATE users SET name = 42, password = 'xxx' WHERE id = 1";
        let res = parse_query(str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_delete_query_with_where_clause() {
        let str0 = "delete from users where user='aaa' and password= 'xxx'";
        let str1 = "delete from users where user=? and password =?";

        let expected0 = "DELETE FROM users WHERE user = 'aaa' AND password = 'xxx'";
        let expected1 = "DELETE FROM users WHERE user = ? AND password = ?";

        let res0 = parse_query(str0);
        let res1 = parse_query(str1);
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

        let res0 = parse_query(str0);
        let res1 = parse_query(str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }
}

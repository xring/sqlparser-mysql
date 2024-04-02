use std::fmt;
use std::io::BufRead;
use std::str;

use das::set_statement::SetStatement;
use nom::combinator::map;
use nom::error::VerboseError;
use nom::{IResult, Offset};

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
use dds::{
    alter_database, alter_table, create_index, create_table, drop_database, drop_event_parser,
    drop_function, drop_index, drop_logfile_group, drop_procedure, drop_server,
    drop_spatial_reference_system, drop_table, drop_tablespace, drop_trigger, drop_view,
    rename_table, truncate_table,
};
use dms::compound_select::CompoundSelectStatement;
use dms::delete::DeleteStatement;
use dms::insert::InsertStatement;
use dms::select::SelectStatement;
use dms::update::UpdateStatement;

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

    // HISTORY
    Insert(InsertStatement),
    CompoundSelect(CompoundSelectStatement),
    Select(SelectStatement),
    Delete(DeleteStatement),
    Update(UpdateStatement),
    Set(SetStatement),
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

const PARSERS: [fn(&str) -> IResult<&str, Statement, VerboseError<&str>>; 18] = [
    |i| map(alter_database, |parsed| Statement::AlterDatabase(parsed))(i),
    |i| map(alter_table, |parsed| Statement::AlterTable(parsed))(i),
    |i| map(create_index, |parsed| Statement::CreateIndex(parsed))(i),
    |i| map(create_table, |parsed| Statement::CreateTable(parsed))(i),
    |i| map(drop_database, |parsed| Statement::DropDatabase(parsed))(i),
    |i| map(drop_event_parser, |parsed| Statement::DropEvent(parsed))(i),
    |i| map(drop_function, |parsed| Statement::DropFunction(parsed))(i),
    |i| map(drop_index, |parsed| Statement::DropIndex(parsed))(i),
    |i| {
        map(drop_logfile_group, |parsed| {
            Statement::DropLogfileGroup(parsed)
        })(i)
    },
    |i| map(drop_procedure, |parsed| Statement::DropProcedure(parsed))(i),
    |i| map(drop_server, |parsed| Statement::DropServer(parsed))(i),
    |i| {
        map(drop_spatial_reference_system, |parsed| {
            Statement::DropSpatialReferenceSystem(parsed)
        })(i)
    },
    |i| map(drop_table, |parsed| Statement::DropTable(parsed))(i),
    |i| map(drop_tablespace, |parsed| Statement::DropTableSpace(parsed))(i),
    |i| map(drop_trigger, |parsed| Statement::DropTrigger(parsed))(i),
    |i| map(drop_view, |parsed| Statement::DropView(parsed))(i),
    |i| map(rename_table, |parsed| Statement::RenameTable(parsed))(i),
    |i| map(truncate_table, |parsed| Statement::TruncateTable(parsed))(i),
];

pub fn parse_sql(input: &str) -> Result<Statement, String> {
    let mut deepest_error = None;
    let mut max_consumed = 0;

    // TODO need parallel parse ?
    for mut parser in PARSERS {
        match parser(input) {
            Ok(result) => return Ok(result.1),
            Err(nom::Err::Error(err)) => {
                let consumed = input.offset(err.errors[0].0);
                if consumed > max_consumed {
                    deepest_error = Some(err.errors[0].0);
                    max_consumed = consumed;
                }
            }
            Err(e) => return Err(String::from("failed to parse sql: other error")),
        }
    }
    let err_msg = deepest_error.unwrap().split(" ").next().unwrap_or("");
    let err_msg = format!(
        "failed to parse sql, error in SQL syntax near `{}`",
        err_msg
    );
    Err(err_msg)
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
        let res = parse_sql(str);
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
    fn trim_query() {
        let str = "   INSERT INTO users VALUES (42, \"test\");     ";
        let res = parse_sql(str);
        assert!(res.is_ok());
    }

    #[test]
    fn parse_byte_slice() {
        let str = "INSERT INTO users VALUES (42, \"test\");";
        let res = parse_sql(&str);
        assert!(res.is_ok());
    }

    #[test]
    fn parse_byte_vector() {
        let str = "INSERT INTO users VALUES (42, \"test\");";
        let res = parse_sql(&str);
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

        let res0 = parse_sql(str0);
        let res1 = parse_sql(str1);
        let res2 = parse_sql(str2);
        let res3 = parse_sql(str3);
        let res4 = parse_sql(str4);
        let res5 = parse_sql(str5);

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

        let res1 = parse_sql(str1);
        let res2 = parse_sql(str2);
        let res3 = parse_sql(str3);

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

        let res0 = parse_sql(str0);
        let res1 = parse_sql(str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn format_select_query_with_function() {
        let str1 = "select count(*) from users";
        let expected1 = "SELECT count(*) FROM users";

        let res1 = parse_sql(str1);
        assert!(res1.is_ok());
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }

    #[test]
    fn display_insert_query() {
        let str = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
        let res = parse_sql(str);
        assert!(res.is_ok());
        assert_eq!(str, format!("{}", res.unwrap()));
    }

    #[test]
    fn display_insert_query_no_columns() {
        let str = "INSERT INTO users VALUES ('aaa', 'xxx')";
        let expected = "INSERT INTO users VALUES ('aaa', 'xxx')";
        let res = parse_sql(str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_insert_query() {
        let str = "insert into users (name, password) values ('aaa', 'xxx')";
        let expected = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
        let res = parse_sql(str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_update_query() {
        let str = "update users set name=42, password='xxx' where id=1";
        let expected = "UPDATE users SET name = 42, password = 'xxx' WHERE id = 1";
        let res = parse_sql(str);
        assert!(res.is_ok());
        assert_eq!(expected, format!("{}", res.unwrap()));
    }

    #[test]
    fn format_delete_query_with_where_clause() {
        let str0 = "delete from users where user='aaa' and password= 'xxx'";
        let str1 = "delete from users where user=? and password =?";

        let expected0 = "DELETE FROM users WHERE user = 'aaa' AND password = 'xxx'";
        let expected1 = "DELETE FROM users WHERE user = ? AND password = ?";

        let res0 = parse_sql(str0);
        let res1 = parse_sql(str1);
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

        let res0 = parse_sql(str0);
        let res1 = parse_sql(str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }
}

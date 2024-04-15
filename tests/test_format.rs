extern crate sqlparser_mysql;

use sqlparser_mysql::{ParseConfig, Parser};

#[test]
fn format_select() {
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
fn format_select_with_where_clause() {
    let str0 = "select name, password from users as u where user='aaa' and password= 'xxx'";
    let str1 = "select name, password from users as u where user=? and password =?";

    let expected0 = "SELECT name, password FROM users AS u WHERE user = 'aaa' AND password = 'xxx'";
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
fn format_select_with_function() {
    let str1 = "select count(*) from users";
    let expected1 = "SELECT count(*) FROM users";
    let config = ParseConfig::default();
    let res1 = Parser::parse(&config, str1);
    assert!(res1.is_ok());
    assert_eq!(expected1, format!("{}", res1.unwrap()));
}

#[test]
fn format_insert() {
    let str = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
    let config = ParseConfig::default();
    let res = Parser::parse(&config, str);
    assert!(res.is_ok());
    assert_eq!(str, format!("{}", res.unwrap()));
}

#[test]
fn format_update() {
    let str = "update users set name=42, password='xxx' where id=1";
    let expected = "UPDATE users SET name = 42, password = 'xxx' WHERE id = 1";
    let config = ParseConfig::default();
    let res = Parser::parse(&config, str);
    assert!(res.is_ok());
    assert_eq!(expected, format!("{}", res.unwrap()));
}

#[test]
fn format_delete_with_where_clause() {
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

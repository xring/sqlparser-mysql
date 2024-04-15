extern crate sqlparser_mysql;

use sqlparser_mysql::base::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
use sqlparser_mysql::base::{Column, FieldValueExpression, ItemPlaceholder, Literal, Table};
use sqlparser_mysql::dms::InsertStatement;
use sqlparser_mysql::{ParseConfig, Parser, Statement};

#[test]
fn simple_insert() {
    let str = "INSERT INTO users VALUES (33, \"test\");";
    let config = ParseConfig::default();
    let res = Parser::parse(&config, str);
    assert!(res.is_ok());

    let expected = Statement::Insert(InsertStatement {
        table: Table::from("users"),
        fields: None,
        data: vec![vec![33.into(), "test".into()]],
        ..Default::default()
    });

    assert_eq!(res.unwrap(), expected);
}

/////////////// INSERT
#[test]
fn format_insert_query() {
    let str = "insert into users (name, password) values ('aaa', 'xxx')";
    let expected = "INSERT INTO users (name, password) VALUES ('aaa', 'xxx')";
    let res = InsertStatement::parse(str);
    println!("{:?}", res);
    assert!(res.is_ok());
    assert_eq!(expected, format!("{}", res.unwrap().1));
}

#[test]
fn trim_query() {
    let str = "   INSERT INTO users VALUES (42, \"test\");     ";
    let res = InsertStatement::parse(str.trim());
    println!("{:?}", res);
    assert!(res.is_ok());
}

#[test]
fn display_insert_query_no_columns() {
    let str = "INSERT INTO users VALUES ('aaa', 'xxx')";
    let expected = "INSERT INTO users VALUES ('aaa', 'xxx')";
    let res = InsertStatement::parse(str);
    assert!(res.is_ok());
    assert_eq!(format!("{}", res.unwrap().1), expected);
}

#[test]
fn on_duplicate() {
    let str = "ON DUPLICATE KEY UPDATE `value` = `value` + 1";
    let res = InsertStatement::on_duplicate(str);
    println!("{:?}", res);
}

#[test]
fn simple_insert_schema() {
    let str = "INSERT INTO db1.users VALUES (42, \"test\");";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from(("db1", "users")),
            fields: None,
            data: vec![vec![42.into(), "test".into()]],
            ..Default::default()
        }
    );
}

#[test]
fn complex_insert() {
    let str = "INSERT INTO users VALUES (42, 'test', \"test\", CURRENT_TIMESTAMP);";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("users"),
            fields: None,
            data: vec![vec![
                42.into(),
                "test".into(),
                "test".into(),
                Literal::CurrentTimestamp,
            ],],
            ..Default::default()
        }
    );
}

#[test]
fn insert_with_field_names() {
    let str = "INSERT INTO users (id, name) VALUES (42, \"test\");";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("users"),
            fields: Some(vec![Column::from("id"), Column::from("name")]),
            data: vec![vec![42.into(), "test".into()]],
            ..Default::default()
        }
    );
}

// Issue #3
#[test]
fn insert_without_spaces() {
    let str = "INSERT INTO users(id, name) VALUES(42, \"test\");";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("users"),
            fields: Some(vec![Column::from("id"), Column::from("name")]),
            data: vec![vec![42.into(), "test".into()]],
            ..Default::default()
        }
    );
}

#[test]
fn multi_insert() {
    let str = "INSERT INTO users (id, name) VALUES (42, \"test\"),(21, \"test2\");";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("users"),
            fields: Some(vec![Column::from("id"), Column::from("name")]),
            data: vec![
                vec![42.into(), "test".into()],
                vec![21.into(), "test2".into()],
            ],
            ..Default::default()
        }
    );
}

#[test]
fn insert_with_parameters() {
    let str = "INSERT INTO users (id, name) VALUES (?, ?);";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("users"),
            fields: Some(vec![Column::from("id"), Column::from("name")]),
            data: vec![vec![
                Literal::Placeholder(ItemPlaceholder::QuestionMark),
                Literal::Placeholder(ItemPlaceholder::QuestionMark),
            ]],
            ..Default::default()
        }
    );
}

#[test]
fn insert_with_on_dup_update() {
    let str = "INSERT INTO keystores (`key`, `value`) VALUES ($1, :2) \
                       ON DUPLICATE KEY UPDATE `value` = `value` + 1";

    let res = InsertStatement::parse(str);

    println!("{:?}", res);

    let expected_ae = ArithmeticExpression::new(
        ArithmeticOperator::Add,
        ArithmeticBase::Column(Column::from("value")),
        ArithmeticBase::Scalar(1.into()),
        None,
    );
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("keystores"),
            fields: Some(vec![Column::from("key"), Column::from("value")]),
            data: vec![vec![
                Literal::Placeholder(ItemPlaceholder::DollarNumber(1)),
                Literal::Placeholder(ItemPlaceholder::ColonNumber(2)),
            ]],
            on_duplicate: Some(vec![(
                Column::from("value"),
                FieldValueExpression::Arithmetic(expected_ae),
            ),]),
            ..Default::default()
        }
    );
}

#[test]
fn insert_with_leading_value_whitespace() {
    let str = "INSERT INTO users (id, name) VALUES ( 42, \"test\");";

    let res = InsertStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        InsertStatement {
            table: Table::from("users"),
            fields: Some(vec![Column::from("id"), Column::from("name")]),
            data: vec![vec![42.into(), "test".into()]],
            ..Default::default()
        }
    );
}

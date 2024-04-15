extern crate sqlparser_mysql;
use sqlparser_mysql::base::condition::ConditionBase::Field;
use sqlparser_mysql::base::condition::ConditionExpression::{Base, ComparisonOp};
use sqlparser_mysql::base::condition::{ConditionBase, ConditionTree};
use sqlparser_mysql::base::{Column, Literal, Operator, Table};
use sqlparser_mysql::dms::DeleteStatement;

/////////////// DELETE
#[test]
fn simple_delete() {
    let str = "DELETE FROM users;";
    let res = DeleteStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        DeleteStatement {
            table: Table::from("users"),
            ..Default::default()
        }
    );
}

#[test]
fn simple_delete_schema() {
    let str = "DELETE FROM db1.users;";
    let res = DeleteStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        DeleteStatement {
            table: Table::from(("db1", "users")),
            ..Default::default()
        }
    );
}

#[test]
fn delete_with_where_clause() {
    let str = "DELETE FROM users WHERE id = 1;";
    let res = DeleteStatement::parse(str);

    let expected_left = Base(Field(Column::from("id")));
    let expected_where_cond = Some(ComparisonOp(ConditionTree {
        left: Box::new(expected_left),
        right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1)))),
        operator: Operator::Equal,
    }));
    assert_eq!(
        res.unwrap().1,
        DeleteStatement {
            table: Table::from("users"),
            where_clause: expected_where_cond,
        }
    );
}

#[test]
fn format_delete() {
    let str = "DELETE FROM users WHERE id = 1";
    let expected = "DELETE FROM users WHERE id = 1";
    let res = DeleteStatement::parse(str);
    assert_eq!(format!("{}", res.unwrap().1), expected);
}

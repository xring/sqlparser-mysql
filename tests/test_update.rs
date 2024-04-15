extern crate sqlparser_mysql;

use sqlparser_mysql::base::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
use sqlparser_mysql::base::condition::ConditionExpression::{Base, ComparisonOp};
use sqlparser_mysql::base::condition::{ConditionBase, ConditionTree};
use sqlparser_mysql::base::{
    Column, FieldValueExpression, ItemPlaceholder, Literal, LiteralExpression, Operator, Real,
    Table,
};
use sqlparser_mysql::dms::UpdateStatement;

/////////////// UPDATE
#[test]
fn simple_update() {
    let str = "UPDATE users SET id = 42, name = 'test'";

    let res = UpdateStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        UpdateStatement {
            table: Table::from("users"),
            fields: vec![
                (
                    Column::from("id"),
                    FieldValueExpression::Literal(LiteralExpression::from(Literal::from(42))),
                ),
                (
                    Column::from("name"),
                    FieldValueExpression::Literal(LiteralExpression::from(Literal::from("test",))),
                ),
            ],
            ..Default::default()
        }
    );
}

#[test]
fn update_with_where_clause() {
    let str = "UPDATE users SET id = 42, name = 'test' WHERE id = 1";

    let res = UpdateStatement::parse(str);
    let expected_left = Base(ConditionBase::Field(Column::from("id")));
    let expected_where_cond = Some(ComparisonOp(ConditionTree {
        left: Box::new(expected_left),
        right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1)))),
        operator: Operator::Equal,
    }));
    assert_eq!(
        res.unwrap().1,
        UpdateStatement {
            table: Table::from("users"),
            fields: vec![
                (
                    Column::from("id"),
                    FieldValueExpression::Literal(LiteralExpression::from(Literal::from(42))),
                ),
                (
                    Column::from("name"),
                    FieldValueExpression::Literal(LiteralExpression::from(Literal::from("test",))),
                ),
            ],
            where_clause: expected_where_cond,
        }
    );
}

#[test]
fn format_update_with_where_clause() {
    let str = "UPDATE users SET id = 42, name = 'test' WHERE id = 1";
    let expected = "UPDATE users SET id = 42, name = 'test' WHERE id = 1";
    let res = UpdateStatement::parse(str);
    assert_eq!(format!("{}", res.unwrap().1), expected);
}

#[test]
fn updated_with_neg_float() {
    let str = "UPDATE `stories` SET `hotness` = -19216.5479744 WHERE `stories`.`id` = ?";

    let res = UpdateStatement::parse(str);
    let expected_left = Base(ConditionBase::Field(Column::from("stories.id")));
    let expected_where_cond = Some(ComparisonOp(ConditionTree {
        left: Box::new(expected_left),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    }));
    assert_eq!(
        res.unwrap().1,
        UpdateStatement {
            table: Table::from("stories"),
            fields: vec![(
                Column::from("hotness"),
                FieldValueExpression::Literal(LiteralExpression::from(Literal::FixedPoint(Real {
                    integral: -19216,
                    fractional: 5479744,
                }),)),
            ),],
            where_clause: expected_where_cond,
        }
    );
}

#[test]
fn update_with_arithmetic_and_where() {
    let str = "UPDATE users SET karma = karma + 1 WHERE users.id = ?;";

    let res = UpdateStatement::parse(str);
    let expected_where_cond = Some(ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("users.id")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    }));
    let expected_ae = ArithmeticExpression::new(
        ArithmeticOperator::Add,
        ArithmeticBase::Column(Column::from("karma")),
        ArithmeticBase::Scalar(1.into()),
        None,
    );
    assert_eq!(
        res.unwrap().1,
        UpdateStatement {
            table: Table::from("users"),
            fields: vec![(
                Column::from("karma"),
                FieldValueExpression::Arithmetic(expected_ae),
            ),],
            where_clause: expected_where_cond,
        }
    );
}

#[test]
fn update_with_arithmetic() {
    let str = "UPDATE users SET karma = karma + 1;";

    let res = UpdateStatement::parse(str);
    let expected_ae = ArithmeticExpression::new(
        ArithmeticOperator::Add,
        ArithmeticBase::Column(Column::from("karma")),
        ArithmeticBase::Scalar(1.into()),
        None,
    );
    assert_eq!(
        res.unwrap().1,
        UpdateStatement {
            table: Table::from("users"),
            fields: vec![(
                Column::from("karma"),
                FieldValueExpression::Arithmetic(expected_ae),
            ),],
            ..Default::default()
        }
    );
}

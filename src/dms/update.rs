use std::{fmt, str};

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::sequence::tuple;
use nom::IResult;

use base::column::Column;
use base::error::ParseSQLError;
use base::table::Table;
use base::FieldValueExpression;
use common::condition::ConditionExpression;
use common::keywords::escape_if_keyword;
use common::statement_terminator;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateStatement {
    pub table: Table,
    pub fields: Vec<(Column, FieldValueExpression)>,
    pub where_clause: Option<ConditionExpression>,
}

impl UpdateStatement {
    pub fn parse(i: &str) -> IResult<&str, UpdateStatement, ParseSQLError<&str>> {
        let (remaining_input, (_, _, table, _, _, _, fields, _, where_clause, _)) = tuple((
            tag_no_case("UPDATE"),
            multispace1,
            Table::table_reference,
            multispace1,
            tag_no_case("SET"),
            multispace1,
            FieldValueExpression::assignment_expr_list,
            multispace0,
            opt(ConditionExpression::parse),
            statement_terminator,
        ))(i)?;
        Ok((
            remaining_input,
            UpdateStatement {
                table,
                fields,
                where_clause,
            },
        ))
    }
}

impl fmt::Display for UpdateStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UPDATE {} ", escape_if_keyword(&self.table.name))?;
        assert!(!self.fields.is_empty());
        write!(
            f,
            "SET {}",
            self.fields
                .iter()
                .map(|(col, literal)| format!("{} = {}", col, literal))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(ref where_clause) = self.where_clause {
            write!(f, " WHERE ")?;
            write!(f, "{}", where_clause)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::{FieldValueExpression, ItemPlaceholder, Literal, LiteralExpression, Operator, Real};
    use common::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
    use common::condition::ConditionBase;
    use common::condition::ConditionExpression::{Base, ComparisonOp};
    use common::condition::ConditionTree;

    use super::*;

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
                        FieldValueExpression::Literal(LiteralExpression::from(Literal::from(
                            "test",
                        ))),
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
                        FieldValueExpression::Literal(LiteralExpression::from(Literal::from(
                            "test",
                        ))),
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
                    FieldValueExpression::Literal(LiteralExpression::from(Literal::FixedPoint(
                        Real {
                            integral: -19216,
                            fractional: 5479744,
                        }
                    ),)),
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
}

use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

use base::arithmetic::ArithmeticExpression;
use base::column::Column;
use base::error::ParseSQLError;
use base::{Literal, Operator};
use dms::{BetweenAndClause, SelectStatement};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ConditionBase {
    Field(Column),
    Literal(Literal),
    LiteralList(Vec<Literal>),
    NestedSelect(Box<SelectStatement>),
}

impl fmt::Display for ConditionBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConditionBase::Field(ref col) => write!(f, "{}", col),
            ConditionBase::Literal(ref literal) => write!(f, "{}", literal),
            ConditionBase::LiteralList(ref ll) => write!(
                f,
                "({})",
                ll.iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ConditionBase::NestedSelect(ref select) => write!(f, "{}", select),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ConditionTree {
    pub operator: Operator,
    pub left: Box<ConditionExpression>,
    pub right: Box<ConditionExpression>,
}

impl<'a> ConditionTree {
    pub fn contained_columns(&'a self) -> HashSet<&'a Column> {
        let mut s = HashSet::new();
        let mut q = VecDeque::<&'a ConditionTree>::new();
        q.push_back(self);
        while let Some(ct) = q.pop_front() {
            match *ct.left.as_ref() {
                ConditionExpression::Base(ConditionBase::Field(ref c)) => {
                    s.insert(c);
                }
                ConditionExpression::LogicalOp(ref ct)
                | ConditionExpression::ComparisonOp(ref ct) => q.push_back(ct),
                _ => (),
            }
            match *ct.right.as_ref() {
                ConditionExpression::Base(ConditionBase::Field(ref c)) => {
                    s.insert(c);
                }
                ConditionExpression::LogicalOp(ref ct)
                | ConditionExpression::ComparisonOp(ref ct) => q.push_back(ct),
                _ => (),
            }
        }
        s
    }
}

impl fmt::Display for ConditionTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.left)?;
        write!(f, " {} ", self.operator)?;
        write!(f, "{}", self.right)
    }
}

/// WHERE CLAUSE
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ConditionExpression {
    ComparisonOp(ConditionTree),
    LogicalOp(ConditionTree),
    NegationOp(Box<ConditionExpression>),
    ExistsOp(Box<SelectStatement>),
    Base(ConditionBase),
    Arithmetic(Box<ArithmeticExpression>),
    Bracketed(Box<ConditionExpression>),
    BetweenAnd(BetweenAndClause),
}

impl ConditionExpression {
    pub fn parse(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, where_condition)) = tuple((
            multispace0,
            tag_no_case("WHERE"),
            multispace1,
            Self::condition_expr,
        ))(i)?;

        Ok((remaining_input, where_condition))
    }

    pub fn having_clause(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, ce)) = tuple((
            multispace0,
            tag_no_case("HAVING"),
            multispace1,
            Self::condition_expr,
        ))(i)?;

        Ok((remaining_input, ce))
    }

    // Parse a conditional expression into a condition tree structure
    pub fn condition_expr(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let cond = map(
            separated_pair(
                Self::and_expr,
                delimited(multispace0, tag_no_case("OR"), multispace1),
                Self::condition_expr,
            ),
            |p| {
                ConditionExpression::LogicalOp(ConditionTree {
                    operator: Operator::Or,
                    left: Box::new(p.0),
                    right: Box::new(p.1),
                })
            },
        );

        alt((cond, Self::and_expr))(i)
    }

    fn and_expr(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let cond = map(
            separated_pair(
                Self::parenthetical_expr,
                delimited(multispace0, tag_no_case("AND"), multispace1),
                Self::and_expr,
            ),
            |p| {
                ConditionExpression::LogicalOp(ConditionTree {
                    operator: Operator::And,
                    left: Box::new(p.0),
                    right: Box::new(p.1),
                })
            },
        );

        alt((cond, Self::parenthetical_expr))(i)
    }

    fn parenthetical_expr_helper(
        i: &str,
    ) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let (remaining_input, (_, _, left_expr, _, _, _, operator, _, right_expr)) = tuple((
            tag("("),
            multispace0,
            Self::simple_expr,
            multispace0,
            tag(")"),
            multispace0,
            Operator::parse,
            multispace0,
            Self::simple_expr,
        ))(i)?;

        let left = Box::new(ConditionExpression::Bracketed(Box::new(left_expr)));
        let right = Box::new(right_expr);
        let cond = ConditionExpression::ComparisonOp(ConditionTree {
            operator,
            left,
            right,
        });

        Ok((remaining_input, cond))
    }

    pub fn parenthetical_expr(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        alt((
            Self::parenthetical_expr_helper,
            map(
                delimited(
                    terminated(tag("("), multispace0),
                    ConditionExpression::condition_expr,
                    delimited(multispace0, tag(")"), multispace0),
                ),
                |inner| ConditionExpression::Bracketed(Box::new(inner)),
            ),
            Self::not_expr,
        ))(i)
    }

    fn not_expr(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        alt((
            map(
                preceded(
                    pair(tag_no_case("NOT"), multispace1),
                    Self::parenthetical_expr,
                ),
                |right| ConditionExpression::NegationOp(Box::new(right)),
            ),
            Self::boolean_primary,
        ))(i)
    }

    fn is_null(i: &str) -> IResult<&str, (Operator, ConditionExpression), ParseSQLError<&str>> {
        let (remaining_input, (_, _, not, _, _)) = tuple((
            tag_no_case("IS"),
            multispace0,
            opt(tag_no_case("NOT")),
            multispace0,
            tag_no_case("NULL"),
        ))(i)?;

        // XXX(malte): bit of a hack; would consumers ever need to know
        // about "IS NULL" vs. "= NULL"?
        Ok((
            remaining_input,
            (
                if not.is_some() {
                    Operator::NotEqual
                } else {
                    Operator::Equal
                },
                ConditionExpression::Base(ConditionBase::Literal(Literal::Null)),
            ),
        ))
    }

    fn in_operation(
        i: &str,
    ) -> IResult<&str, (Operator, ConditionExpression), ParseSQLError<&str>> {
        map(
            separated_pair(
                opt(terminated(tag_no_case("NOT"), multispace1)),
                terminated(tag_no_case("IN"), multispace0),
                alt((
                    map(
                        delimited(tag("("), SelectStatement::nested_selection, tag(")")),
                        |s| ConditionBase::NestedSelect(Box::new(s)),
                    ),
                    map(delimited(tag("("), Literal::value_list, tag(")")), |vs| {
                        ConditionBase::LiteralList(vs)
                    }),
                )),
            ),
            |p| {
                let nested = ConditionExpression::Base(p.1);
                if (p.0).is_some() {
                    (Operator::NotIn, nested)
                } else {
                    (Operator::In, nested)
                }
            },
        )(i)
    }

    fn boolean_primary_rest(
        i: &str,
    ) -> IResult<&str, (Operator, ConditionExpression), ParseSQLError<&str>> {
        alt((
            Self::is_null,
            Self::in_operation,
            separated_pair(Operator::parse, multispace0, Self::predicate),
        ))(i)
    }

    fn boolean_primary(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        alt((
            map(
                separated_pair(Self::predicate, multispace0, Self::boolean_primary_rest),
                |e: (ConditionExpression, (Operator, ConditionExpression))| {
                    ConditionExpression::ComparisonOp(ConditionTree {
                        operator: (e.1).0,
                        left: Box::new(e.0),
                        right: Box::new((e.1).1),
                    })
                },
            ),
            Self::predicate,
        ))(i)
    }

    fn predicate(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let nested_exists = map(
            tuple((
                opt(delimited(multispace0, tag_no_case("NOT"), multispace1)),
                delimited(multispace0, tag_no_case("EXISTS"), multispace0),
                delimited(
                    pair(tag("("), multispace0),
                    SelectStatement::nested_selection,
                    pair(multispace0, tag(")")),
                ),
            )),
            |p| {
                let nested = ConditionExpression::ExistsOp(Box::new(p.2));
                if (p.0).is_some() {
                    ConditionExpression::NegationOp(Box::new(nested))
                } else {
                    nested
                }
            },
        );

        alt((Self::simple_expr, nested_exists))(i)
    }

    pub fn simple_expr(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        let simple_expr = alt((
            map(
                delimited(
                    terminated(tag("("), multispace0),
                    ArithmeticExpression::parse,
                    preceded(multispace0, tag(")")),
                ),
                |e| {
                    ConditionExpression::Bracketed(Box::new(ConditionExpression::Arithmetic(
                        Box::new(e),
                    )))
                },
            ),
            map(ArithmeticExpression::parse, |e| {
                ConditionExpression::Arithmetic(Box::new(e))
            }),
            map(Literal::parse, |lit| {
                ConditionExpression::Base(ConditionBase::Literal(lit))
            }),
            map(Column::parse, |f| {
                ConditionExpression::Base(ConditionBase::Field(f))
            }),
            map(
                delimited(tag("("), SelectStatement::nested_selection, tag(")")),
                |s| ConditionExpression::Base(ConditionBase::NestedSelect(Box::new(s))),
            ),
        ));

        alt((Self::between_and, simple_expr))(i)
    }

    fn between_and(i: &str) -> IResult<&str, ConditionExpression, ParseSQLError<&str>> {
        map(BetweenAndClause::parse, |x| {
            ConditionExpression::BetweenAnd(x)
        })(i)
    }
}

impl fmt::Display for ConditionExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConditionExpression::ComparisonOp(ref tree) => write!(f, "{}", tree),
            ConditionExpression::LogicalOp(ref tree) => write!(f, "{}", tree),
            ConditionExpression::NegationOp(ref expr) => write!(f, "NOT {}", expr),
            ConditionExpression::ExistsOp(ref expr) => write!(f, "EXISTS {}", expr),
            ConditionExpression::Bracketed(ref expr) => write!(f, "({})", expr),
            ConditionExpression::Base(ref base) => write!(f, "{}", base),
            ConditionExpression::Arithmetic(ref expr) => write!(f, "{}", expr),
            ConditionExpression::BetweenAnd(ref expr) => write!(f, "{}", expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use base::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
    use base::condition::ConditionBase::{Field, LiteralList, NestedSelect};
    use base::condition::ConditionExpression::{
        Base, Bracketed, ComparisonOp, LogicalOp, NegationOp,
    };
    use base::table::Table;
    use base::{FieldDefinitionExpression, ItemPlaceholder};

    use super::*;

    fn flat_condition_tree(
        op: Operator,
        l: ConditionBase,
        r: ConditionBase,
    ) -> ConditionExpression {
        ConditionExpression::ComparisonOp(ConditionTree {
            operator: op,
            left: Box::new(ConditionExpression::Base(l)),
            right: Box::new(ConditionExpression::Base(r)),
        })
    }

    #[test]
    fn ct_contained_columns() {
        use std::collections::HashSet;

        let cond = "a.foo = ? and b.bar = 42";

        let res = ConditionExpression::condition_expr(cond);
        let c1 = Column::from("a.foo");
        let c2 = Column::from("b.bar");
        let mut expected_cols = HashSet::new();
        expected_cols.insert(&c1);
        expected_cols.insert(&c2);
        match res.unwrap().1 {
            ConditionExpression::LogicalOp(ct) => {
                assert_eq!(ct.contained_columns(), expected_cols);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn equality_placeholder() {
        x_equality_variable_placeholder(
            "foo = ?",
            Literal::Placeholder(ItemPlaceholder::QuestionMark),
        );
    }

    #[test]
    fn equality_variable_placeholder() {
        x_equality_variable_placeholder(
            "foo = :12",
            Literal::Placeholder(ItemPlaceholder::ColonNumber(12)),
        );
    }

    #[test]
    fn equality_variable_placeholder_with_dollar_sign() {
        x_equality_variable_placeholder(
            "foo = $12",
            Literal::Placeholder(ItemPlaceholder::DollarNumber(12)),
        );
    }

    fn x_equality_variable_placeholder(cond: &str, literal: Literal) {
        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(
            res.unwrap().1,
            flat_condition_tree(
                Operator::Equal,
                ConditionBase::Field(Column::from("foo")),
                ConditionBase::Literal(literal),
            )
        );
    }

    fn x_operator_value(op: ArithmeticOperator, value: Literal) -> ConditionExpression {
        ConditionExpression::Arithmetic(Box::new(ArithmeticExpression::new(
            op,
            ArithmeticBase::Column(Column::from("x")),
            ArithmeticBase::Scalar(value),
            None,
        )))
    }

    #[test]
    fn simple_arithmetic_expression() {
        let cond = "x + 3";

        let res = ConditionExpression::simple_expr(cond);
        assert_eq!(
            res.unwrap().1,
            x_operator_value(ArithmeticOperator::Add, 3.into())
        );
    }

    #[test]
    fn simple_arithmetic_expression_with_parenthesis() {
        let cond = "( x - 2 )";

        let res = ConditionExpression::simple_expr(cond);
        assert_eq!(
            res.unwrap().1,
            ConditionExpression::Bracketed(Box::new(x_operator_value(
                ArithmeticOperator::Subtract,
                2.into(),
            )))
        );
    }

    #[test]
    fn parenthetical_arithmetic_expression() {
        let cond = "( x * 5 )";

        let res = ConditionExpression::parenthetical_expr(cond);
        assert_eq!(
            res.unwrap().1,
            ConditionExpression::Bracketed(Box::new(x_operator_value(
                ArithmeticOperator::Multiply,
                5.into(),
            )))
        );
    }

    #[test]
    fn condition_expression_with_arithmetics() {
        let cond = "x * 3 = 21";

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(
            res.unwrap().1,
            ConditionExpression::ComparisonOp(ConditionTree {
                operator: Operator::Equal,
                left: Box::new(x_operator_value(ArithmeticOperator::Multiply, 3.into())),
                right: Box::new(ConditionExpression::Base(ConditionBase::Literal(21.into()))),
            })
        );
    }

    #[test]
    fn condition_expression_with_arithmetics_and_parenthesis() {
        let cond = "(x - 7 = 15)";

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(
            res.unwrap().1,
            ConditionExpression::Bracketed(Box::new(ConditionExpression::ComparisonOp(
                ConditionTree {
                    operator: Operator::Equal,
                    left: Box::new(x_operator_value(ArithmeticOperator::Subtract, 7.into())),
                    right: Box::new(ConditionExpression::Base(ConditionBase::Literal(15.into()))),
                }
            )))
        );
    }

    #[test]
    fn condition_expression_with_arithmetics_in_parenthesis() {
        let cond = "( x + 2) = 15";

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(
            res.unwrap().1,
            ConditionExpression::ComparisonOp(ConditionTree {
                operator: Operator::Equal,
                left: Box::new(ConditionExpression::Bracketed(Box::new(x_operator_value(
                    ArithmeticOperator::Add,
                    2.into(),
                )))),
                right: Box::new(ConditionExpression::Base(ConditionBase::Literal(15.into()))),
            })
        );
    }

    #[test]
    fn condition_expression_with_arithmetics_in_parenthesis_in_both_side() {
        let cond = "( x + 2) =(x*3)";

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(
            res.unwrap().1,
            ConditionExpression::ComparisonOp(ConditionTree {
                operator: Operator::Equal,
                left: Box::new(ConditionExpression::Bracketed(Box::new(x_operator_value(
                    ArithmeticOperator::Add,
                    2.into(),
                )))),
                right: Box::new(ConditionExpression::Bracketed(Box::new(x_operator_value(
                    ArithmeticOperator::Multiply,
                    3.into(),
                )))),
            })
        );
    }

    #[test]
    fn equality_literals() {
        let cond1 = "foo = 42";
        let cond2 = "foo = \"hello\"";

        let res1 = ConditionExpression::condition_expr(cond1);
        assert_eq!(
            res1.unwrap().1,
            flat_condition_tree(
                Operator::Equal,
                ConditionBase::Field(Column::from("foo")),
                ConditionBase::Literal(Literal::Integer(42)),
            )
        );

        let res2 = ConditionExpression::condition_expr(cond2);
        assert_eq!(
            res2.unwrap().1,
            flat_condition_tree(
                Operator::Equal,
                ConditionBase::Field(Column::from("foo")),
                ConditionBase::Literal(Literal::String(String::from("hello"))),
            )
        );
    }

    #[test]
    fn inequality_literals() {
        let cond1 = "foo >= 42";
        let cond2 = "foo <= 5";

        let res1 = ConditionExpression::condition_expr(cond1);
        assert_eq!(
            res1.unwrap().1,
            flat_condition_tree(
                Operator::GreaterOrEqual,
                ConditionBase::Field(Column::from("foo")),
                ConditionBase::Literal(Literal::Integer(42)),
            )
        );

        let res2 = ConditionExpression::condition_expr(cond2);
        assert_eq!(
            res2.unwrap().1,
            flat_condition_tree(
                Operator::LessOrEqual,
                ConditionBase::Field(Column::from("foo")),
                ConditionBase::Literal(Literal::Integer(5)),
            )
        );
    }

    #[test]
    fn empty_string_literal() {
        let cond = "foo = ''";

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(
            res.unwrap().1,
            flat_condition_tree(
                Operator::Equal,
                ConditionBase::Field(Column::from("foo")),
                ConditionBase::Literal(Literal::String(String::from(""))),
            )
        );
    }

    #[test]
    fn parenthesis() {
        let cond = "(foo = ? or bar = 12) and foobar = 'a'";

        let a = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("foo".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
        });

        let b = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("bar".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(12.into())))),
        });

        let left = Bracketed(Box::new(LogicalOp(ConditionTree {
            operator: Operator::Or,
            left: Box::new(a),
            right: Box::new(b),
        })));

        let right = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("foobar".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::String("a".into())))),
        });

        let complete = LogicalOp(ConditionTree {
            operator: Operator::And,
            left: Box::new(left),
            right: Box::new(right),
        });

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(res.unwrap().1, complete);
    }

    #[test]
    fn order_of_operations() {
        let cond = "foo = ? and bar = 12 or foobar = 'a'";

        let a = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("foo".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
        });

        let b = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("bar".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(12.into())))),
        });

        let left = LogicalOp(ConditionTree {
            operator: Operator::And,
            left: Box::new(a),
            right: Box::new(b),
        });

        let right = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("foobar".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::String("a".into())))),
        });

        let complete = LogicalOp(ConditionTree {
            operator: Operator::Or,
            left: Box::new(left),
            right: Box::new(right),
        });

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(res.unwrap().1, complete);
    }

    #[test]
    fn negation() {
        let cond = "not bar = 12 or foobar = 'a'";
        use base::Literal;

        let left = NegationOp(Box::new(ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("bar".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(12.into())))),
        })));

        let right = ComparisonOp(ConditionTree {
            operator: Operator::Equal,
            left: Box::new(Base(Field("foobar".into()))),
            right: Box::new(Base(ConditionBase::Literal(Literal::String("a".into())))),
        });

        let complete = LogicalOp(ConditionTree {
            operator: Operator::Or,
            left: Box::new(left),
            right: Box::new(right),
        });

        let res = ConditionExpression::condition_expr(cond);
        assert_eq!(res.unwrap().1, complete);
    }

    #[test]
    fn nested_select() {
        use std::default::Default;

        let cond = "bar in (select col from foo)";

        let res = ConditionExpression::condition_expr(cond);

        let nested_select = Box::new(SelectStatement {
            tables: vec![Table::from("foo")],
            fields: FieldDefinitionExpression::from_column_str(&["col"]),
            ..Default::default()
        });

        let expected = flat_condition_tree(
            Operator::In,
            Field("bar".into()),
            NestedSelect(nested_select),
        );

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn exists_in_select() {
        use base::table::Table;
        use std::default::Default;

        let cond = "exists (  select col from foo  )";

        let res = ConditionExpression::condition_expr(cond);

        let nested_select = Box::new(SelectStatement {
            tables: vec![Table::from("foo")],
            fields: FieldDefinitionExpression::from_column_str(&["col"]),
            ..Default::default()
        });

        let expected = ConditionExpression::ExistsOp(nested_select);

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn not_exists_in_select() {
        use base::table::Table;
        use std::default::Default;

        let cond = "not exists (select col from foo)";

        let res = ConditionExpression::condition_expr(cond);

        let nested_select = Box::new(SelectStatement {
            tables: vec![Table::from("foo")],
            fields: FieldDefinitionExpression::from_column_str(&["col"]),
            ..Default::default()
        });

        let expected =
            ConditionExpression::NegationOp(Box::new(ConditionExpression::ExistsOp(nested_select)));

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn and_with_nested_select() {
        use base::table::Table;
        use std::default::Default;

        let cond = "paperId in (select paperId from PaperConflict) and size > 0";

        let res = ConditionExpression::condition_expr(cond);

        let nested_select = Box::new(SelectStatement {
            tables: vec![Table::from("PaperConflict")],
            fields: FieldDefinitionExpression::from_column_str(&["paperId"]),
            ..Default::default()
        });

        let left = flat_condition_tree(
            Operator::In,
            Field("paperId".into()),
            NestedSelect(nested_select),
        );

        let right = flat_condition_tree(
            Operator::Greater,
            Field("size".into()),
            ConditionBase::Literal(0.into()),
        );

        let expected = ConditionExpression::LogicalOp(ConditionTree {
            left: Box::new(left),
            right: Box::new(right),
            operator: Operator::And,
        });

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn in_list_of_values() {
        let cond = "bar in (0)";

        let res = ConditionExpression::condition_expr(cond);

        let expected = flat_condition_tree(
            Operator::In,
            Field("bar".into()),
            LiteralList(vec![0.into()]),
        );

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn is_null() {
        use base::Literal;

        let cond = "bar IS NULL";

        let res = ConditionExpression::condition_expr(cond);
        let expected = flat_condition_tree(
            Operator::Equal,
            Field("bar".into()),
            ConditionBase::Literal(Literal::Null),
        );
        assert_eq!(res.unwrap().1, expected);

        let cond = "bar IS NOT NULL";

        let res = ConditionExpression::condition_expr(cond);
        let expected = flat_condition_tree(
            Operator::NotEqual,
            Field("bar".into()),
            ConditionBase::Literal(Literal::Null),
        );
        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn complex_bracketing() {
        use base::Literal;

        let cond = "`read_ribbons`.`is_following` = 1 \
                    AND `comments`.`user_id` <> `read_ribbons`.`user_id` \
                    AND `saldo` >= 0 \
                    AND ( `parent_comments`.`user_id` = `read_ribbons`.`user_id` \
                    OR ( `parent_comments`.`user_id` IS NULL \
                    AND `stories`.`user_id` = `read_ribbons`.`user_id` ) ) \
                    AND ( `parent_comments`.`id` IS NULL \
                    OR `saldo` >= 0 ) \
                    AND `read_ribbons`.`user_id` = ?";

        let res = ConditionExpression::condition_expr(cond);
        let expected = ConditionExpression::LogicalOp(ConditionTree {
            operator: Operator::And,
            left: Box::new(flat_condition_tree(
                Operator::Equal,
                Field("read_ribbons.is_following".into()),
                ConditionBase::Literal(Literal::Integer(1.into())),
            )),
            right: Box::new(ConditionExpression::LogicalOp(ConditionTree {
                operator: Operator::And,
                left: Box::new(flat_condition_tree(
                    Operator::NotEqual,
                    Field("comments.user_id".into()),
                    Field("read_ribbons.user_id".into()),
                )),
                right: Box::new(ConditionExpression::LogicalOp(ConditionTree {
                    operator: Operator::And,
                    left: Box::new(flat_condition_tree(
                        Operator::GreaterOrEqual,
                        Field("saldo".into()),
                        ConditionBase::Literal(Literal::Integer(0.into())),
                    )),
                    right: Box::new(ConditionExpression::LogicalOp(ConditionTree {
                        operator: Operator::And,
                        left: Box::new(ConditionExpression::Bracketed(Box::new(
                            ConditionExpression::LogicalOp(ConditionTree {
                                operator: Operator::Or,
                                left: Box::new(flat_condition_tree(
                                    Operator::Equal,
                                    Field("parent_comments.user_id".into()),
                                    Field("read_ribbons.user_id".into()),
                                )),
                                right: Box::new(ConditionExpression::Bracketed(Box::new(
                                    ConditionExpression::LogicalOp(ConditionTree {
                                        operator: Operator::And,
                                        left: Box::new(flat_condition_tree(
                                            Operator::Equal,
                                            Field("parent_comments.user_id".into()),
                                            ConditionBase::Literal(Literal::Null),
                                        )),
                                        right: Box::new(flat_condition_tree(
                                            Operator::Equal,
                                            Field("stories.user_id".into()),
                                            Field("read_ribbons.user_id".into()),
                                        )),
                                    }),
                                ))),
                            }),
                        ))),
                        right: Box::new(ConditionExpression::LogicalOp(ConditionTree {
                            operator: Operator::And,
                            left: Box::new(ConditionExpression::Bracketed(Box::new(
                                ConditionExpression::LogicalOp(ConditionTree {
                                    operator: Operator::Or,
                                    left: Box::new(flat_condition_tree(
                                        Operator::Equal,
                                        Field("parent_comments.id".into()),
                                        ConditionBase::Literal(Literal::Null),
                                    )),
                                    right: Box::new(flat_condition_tree(
                                        Operator::GreaterOrEqual,
                                        Field("saldo".into()),
                                        ConditionBase::Literal(Literal::Integer(0)),
                                    )),
                                }),
                            ))),
                            right: Box::new(flat_condition_tree(
                                Operator::Equal,
                                Field("read_ribbons.user_id".into()),
                                ConditionBase::Literal(Literal::Placeholder(
                                    ItemPlaceholder::QuestionMark,
                                )),
                            )),
                        })),
                    })),
                })),
            })),
        });
        let res = res.unwrap().1;
        assert_eq!(res, expected);
    }

    #[test]
    fn not_in_comparison() {
        let qs1 = "id not in (1,2)";
        let res1 = ConditionExpression::condition_expr(qs1);

        let c1 = res1.unwrap().1;
        let expected1 = flat_condition_tree(
            Operator::NotIn,
            Field("id".into()),
            LiteralList(vec![1.into(), 2.into()]),
        );
        assert_eq!(c1, expected1);

        let expected1 = "id NOT IN (1, 2)";
        assert_eq!(format!("{}", c1), expected1);
    }
}

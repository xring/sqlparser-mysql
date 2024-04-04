use std::fmt;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use common::{opt_delimited, statement_terminator, OrderClause};
use dms::select::{LimitClause, SelectStatement};

// TODO 用于 create 语句的 select
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct CompoundSelectStatement {
    pub selects: Vec<(Option<CompoundSelectOperator>, SelectStatement)>,
    pub order: Option<OrderClause>,
    pub limit: Option<LimitClause>,
}

impl CompoundSelectStatement {
    // Parse compound selection
    pub fn parse(i: &str) -> IResult<&str, CompoundSelectStatement, ParseSQLError<&str>> {
        let (remaining_input, (first_select, other_selects, _, order, limit, _)) = tuple((
            opt_delimited(tag("("), SelectStatement::nested_selection, tag(")")),
            many1(Self::other_selects),
            multispace0,
            opt(OrderClause::parse),
            opt(LimitClause::parse),
            statement_terminator,
        ))(i)?;

        let mut selects = vec![(None, first_select)];
        selects.extend(other_selects);

        Ok((
            remaining_input,
            CompoundSelectStatement {
                selects,
                order,
                limit,
            },
        ))
    }

    fn other_selects(
        i: &str,
    ) -> IResult<&str, (Option<CompoundSelectOperator>, SelectStatement), ParseSQLError<&str>> {
        let (remaining_input, (_, op, _, select)) = tuple((
            multispace0,
            CompoundSelectOperator::parse,
            multispace1,
            opt_delimited(
                tag("("),
                delimited(multispace0, SelectStatement::nested_selection, multispace0),
                tag(")"),
            ),
        ))(i)?;

        Ok((remaining_input, (Some(op), select)))
    }
}

impl fmt::Display for CompoundSelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (ref op, ref sel) in &self.selects {
            if op.is_some() {
                write!(f, " {}", op.as_ref().unwrap())?;
            }
            write!(f, " {}", sel)?;
        }
        if self.order.is_some() {
            write!(f, " {}", self.order.as_ref().unwrap())?;
        }
        if self.limit.is_some() {
            write!(f, " {}", self.order.as_ref().unwrap())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum CompoundSelectOperator {
    Union,
    DistinctUnion,
    Intersect,
    Except,
}

impl CompoundSelectOperator {
    // Parse compound operator
    fn parse(i: &str) -> IResult<&str, CompoundSelectOperator, ParseSQLError<&str>> {
        alt((
            map(
                preceded(
                    tag_no_case("UNION"),
                    opt(preceded(
                        multispace1,
                        alt((
                            map(tag_no_case("ALL"), |_| false),
                            map(tag_no_case("DISTINCT"), |_| true),
                        )),
                    )),
                ),
                |distinct| match distinct {
                    // DISTINCT is the default in both MySQL and SQLite
                    None => CompoundSelectOperator::DistinctUnion,
                    Some(d) => {
                        if d {
                            CompoundSelectOperator::DistinctUnion
                        } else {
                            CompoundSelectOperator::Union
                        }
                    }
                },
            ),
            map(tag_no_case("INTERSECT"), |_| {
                CompoundSelectOperator::Intersect
            }),
            map(tag_no_case("EXCEPT"), |_| CompoundSelectOperator::Except),
        ))(i)
    }
}

impl fmt::Display for CompoundSelectOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CompoundSelectOperator::Union => write!(f, "UNION"),
            CompoundSelectOperator::DistinctUnion => write!(f, "UNION DISTINCT"),
            CompoundSelectOperator::Intersect => write!(f, "INTERSECT"),
            CompoundSelectOperator::Except => write!(f, "EXCEPT"),
        }
    }
}

#[cfg(test)]
mod tests {
    use base::column::Column;
    use base::table::Table;
    use base::{FieldDefinitionExpression, FieldValueExpression, Literal};
    use dms::select::SelectStatement;

    use super::*;

    #[test]
    fn simple() {
        let sql = "SELECT * FROM my_table WHERE age < 30;";
        let res = CompoundSelectStatement::parse(sql);
        println!("{:?}", res);
    }

    #[test]
    fn union() {
        let qstr = "SELECT id, 1 FROM Vote UNION SELECT id, stars from Rating;";
        let qstr2 = "(SELECT id, 1 FROM Vote) UNION (SELECT id, stars from Rating);";
        let res = CompoundSelectStatement::parse(qstr);
        let res2 = CompoundSelectStatement::parse(qstr2);

        let first_select = SelectStatement {
            tables: vec![Table::from("Vote")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("id")),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Integer(1).into(),
                )),
            ],
            ..Default::default()
        };
        let second_select = SelectStatement {
            tables: vec![Table::from("Rating")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("id")),
                FieldDefinitionExpression::Col(Column::from("stars")),
            ],
            ..Default::default()
        };
        let expected = CompoundSelectStatement {
            selects: vec![
                (None, first_select),
                (Some(CompoundSelectOperator::DistinctUnion), second_select),
            ],
            order: None,
            limit: None,
        };

        assert_eq!(res.unwrap().1, expected);
        assert_eq!(res2.unwrap().1, expected);
    }

    #[test]
    fn union_strict() {
        let qstr = "SELECT id, 1 FROM Vote);";
        let qstr2 = "(SELECT id, 1 FROM Vote;";
        let qstr3 = "SELECT id, 1 FROM Vote) UNION (SELECT id, stars from Rating;";
        let res = CompoundSelectStatement::parse(qstr);
        let res2 = CompoundSelectStatement::parse(qstr2);
        let res3 = CompoundSelectStatement::parse(qstr3);

        assert!(&res.is_err());
        // assert_eq!(
        //     res.unwrap_err(),
        //     nom::Err::Error(nom::error::Error::new(");", nom::error::ErrorKind::Tag))
        // );
        assert!(&res2.is_err());
        // assert_eq!(
        //     res2.unwrap_err(),
        //     nom::Err::Error(nom::error::Error::new(";", nom::error::ErrorKind::Tag))
        // );
        assert!(&res3.is_err());
        // assert_eq!(
        //     res3.unwrap_err(),
        //     nom::Err::Error(nom::error::Error::new(
        //         ") UNION (SELECT id, stars from Rating;",
        //         nom::error::ErrorKind::Tag,
        //     ))
        // );
    }

    #[test]
    fn multi_union() {
        let qstr = "SELECT id, 1 FROM Vote \
                    UNION SELECT id, stars from Rating \
                    UNION DISTINCT SELECT 42, 5 FROM Vote;";
        let res = CompoundSelectStatement::parse(qstr);

        let first_select = SelectStatement {
            tables: vec![Table::from("Vote")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("id")),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Integer(1).into(),
                )),
            ],
            ..Default::default()
        };
        let second_select = SelectStatement {
            tables: vec![Table::from("Rating")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("id")),
                FieldDefinitionExpression::Col(Column::from("stars")),
            ],
            ..Default::default()
        };
        let third_select = SelectStatement {
            tables: vec![Table::from("Vote")],
            fields: vec![
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Integer(42).into(),
                )),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Integer(5).into(),
                )),
            ],
            ..Default::default()
        };

        let expected = CompoundSelectStatement {
            selects: vec![
                (None, first_select),
                (Some(CompoundSelectOperator::DistinctUnion), second_select),
                (Some(CompoundSelectOperator::DistinctUnion), third_select),
            ],
            order: None,
            limit: None,
        };

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn union_all() {
        let qstr = "SELECT id, 1 FROM Vote UNION ALL SELECT id, stars from Rating;";
        let res = CompoundSelectStatement::parse(qstr);

        let first_select = SelectStatement {
            tables: vec![Table::from("Vote")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("id")),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Integer(1).into(),
                )),
            ],
            ..Default::default()
        };
        let second_select = SelectStatement {
            tables: vec![Table::from("Rating")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("id")),
                FieldDefinitionExpression::Col(Column::from("stars")),
            ],
            ..Default::default()
        };
        let expected = CompoundSelectStatement {
            selects: vec![
                (None, first_select),
                (Some(CompoundSelectOperator::Union), second_select),
            ],
            order: None,
            limit: None,
        };

        assert_eq!(res.unwrap().1, expected);
    }
}

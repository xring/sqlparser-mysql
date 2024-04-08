use std::fmt;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use base::column::Column;
use base::condition::ConditionExpression;
use base::error::ParseSQLError;
use base::table::Table;
use base::CommonParser;
use dms::SelectStatement;

/// parse `join ...` part
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct JoinClause {
    pub operator: JoinOperator,
    pub right: JoinRightSide,
    pub constraint: JoinConstraint,
}

impl JoinClause {
    pub fn parse(i: &str) -> IResult<&str, JoinClause, ParseSQLError<&str>> {
        let (remaining_input, (_, _natural, operator, _, right, _, constraint)) = tuple((
            multispace0,
            opt(terminated(tag_no_case("NATURAL"), multispace1)),
            JoinOperator::parse,
            multispace1,
            JoinRightSide::parse,
            multispace1,
            JoinConstraint::parse,
        ))(i)?;

        Ok((
            remaining_input,
            JoinClause {
                operator,
                right,
                constraint,
            },
        ))
    }
}

impl fmt::Display for JoinClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operator)?;
        write!(f, " {}", self.right)?;
        write!(f, " {}", self.constraint)?;
        Ok(())
    }
}

/// right side of a [JoinOperator]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum JoinRightSide {
    /// A single table.
    Table(Table),
    /// A comma-separated (and implicitly joined) sequence of tables.
    Tables(Vec<Table>),
    /// A nested selection, represented as (query, alias).
    NestedSelect(Box<SelectStatement>, Option<String>),
    /// A nested join clause.
    NestedJoin(Box<JoinClause>),
}

impl JoinRightSide {
    pub fn parse(i: &str) -> IResult<&str, JoinRightSide, ParseSQLError<&str>> {
        let nested_select = map(
            tuple((
                delimited(tag("("), SelectStatement::nested_selection, tag(")")),
                opt(CommonParser::as_alias),
            )),
            |t| JoinRightSide::NestedSelect(Box::new(t.0), t.1.map(String::from)),
        );
        let nested_join = map(delimited(tag("("), JoinClause::parse, tag(")")), |nj| {
            JoinRightSide::NestedJoin(Box::new(nj))
        });
        let table = map(Table::table_reference, JoinRightSide::Table);
        let tables = map(delimited(tag("("), Table::table_list, tag(")")), |tables| {
            JoinRightSide::Tables(tables)
        });
        alt((nested_select, nested_join, table, tables))(i)
    }
}

impl fmt::Display for JoinRightSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JoinRightSide::Table(ref t) => write!(f, "{}", t)?,
            JoinRightSide::NestedSelect(ref q, ref a) => {
                write!(f, "({})", q)?;
                if a.is_some() {
                    write!(f, " AS {}", a.as_ref().unwrap())?;
                }
            }
            JoinRightSide::NestedJoin(ref jc) => write!(f, "({})", jc)?,
            _ => unimplemented!(),
        }
        Ok(())
    }
}

/// join types
/// - join
/// - left join
/// - left outer join
/// - right join
/// - inner join
/// - cross join
/// - straight join
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum JoinOperator {
    Join,
    LeftJoin,
    LeftOuterJoin,
    RightJoin,
    InnerJoin,
    CrossJoin,
    StraightJoin,
}

impl JoinOperator {
    // Parse binary comparison operators
    pub fn parse(i: &str) -> IResult<&str, JoinOperator, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("JOIN"), |_| JoinOperator::Join),
            map(
                tuple((tag_no_case("LEFT"), multispace1, tag_no_case("JOIN"))),
                |_| JoinOperator::LeftJoin,
            ),
            map(
                tuple((
                    tag_no_case("LEFT"),
                    multispace1,
                    tag_no_case("OUTER"),
                    multispace1,
                    tag_no_case("JOIN"),
                )),
                |_| JoinOperator::LeftOuterJoin,
            ),
            map(
                tuple((tag_no_case("RIGHT"), multispace1, tag_no_case("JOIN"))),
                |_| JoinOperator::RightJoin,
            ),
            map(
                tuple((tag_no_case("INNER"), multispace1, tag_no_case("JOIN"))),
                |_| JoinOperator::InnerJoin,
            ),
            map(
                tuple((tag_no_case("CROSS"), multispace1, tag_no_case("JOIN"))),
                |_| JoinOperator::CrossJoin,
            ),
            map(tag_no_case("STRAIGHT_JOIN"), |_| JoinOperator::StraightJoin),
        ))(i)
    }
}

impl fmt::Display for JoinOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JoinOperator::Join => write!(f, "JOIN")?,
            JoinOperator::LeftJoin => write!(f, "LEFT JOIN")?,
            JoinOperator::LeftOuterJoin => write!(f, "LEFT OUTER JOIN")?,
            JoinOperator::RightJoin => write!(f, "RIGHT JOIN")?,
            JoinOperator::InnerJoin => write!(f, "INNER JOIN")?,
            JoinOperator::CrossJoin => write!(f, "CROSS JOIN")?,
            JoinOperator::StraightJoin => write!(f, "STRAIGHT JOIN")?,
        }
        Ok(())
    }
}

/// join constraint
/// - on xxx
/// - using xxx
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum JoinConstraint {
    On(ConditionExpression),
    Using(Vec<Column>),
}

impl JoinConstraint {
    pub fn parse(i: &str) -> IResult<&str, JoinConstraint, ParseSQLError<&str>> {
        let using_clause = map(
            tuple((
                tag_no_case("USING"),
                multispace1,
                delimited(
                    terminated(tag("("), multispace0),
                    Column::field_list,
                    preceded(multispace0, tag(")")),
                ),
            )),
            |t| JoinConstraint::Using(t.2),
        );

        let on_condition = alt((
            delimited(
                terminated(tag("("), multispace0),
                ConditionExpression::condition_expr,
                preceded(multispace0, tag(")")),
            ),
            ConditionExpression::condition_expr,
        ));
        let on_clause = map(tuple((tag_no_case("ON"), multispace1, on_condition)), |t| {
            JoinConstraint::On(t.2)
        });

        alt((using_clause, on_clause))(i)
    }
}

impl fmt::Display for JoinConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JoinConstraint::On(ref ce) => write!(f, "ON {}", ce)?,
            JoinConstraint::Using(ref columns) => write!(
                f,
                "USING ({})",
                columns
                    .iter()
                    .map(|c| format!("{}", c))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::condition::ConditionBase::Field;
    use base::condition::ConditionExpression::Base;
    use base::condition::{ConditionExpression, ConditionTree};
    use base::Operator;

    use super::*;

    #[test]
    fn parse_join() {
        let str = "INNER JOIN tagging ON tags.id = tagging.tag_id";
        let res = JoinClause::parse(str);

        let ct = ConditionTree {
            left: Box::new(Base(Field(Column::from("tags.id")))),
            right: Box::new(Base(Field(Column::from("tagging.tag_id")))),
            operator: Operator::Equal,
        };
        let join_cond = ConditionExpression::ComparisonOp(ct);
        let join = JoinClause {
            operator: JoinOperator::InnerJoin,
            right: JoinRightSide::Table(Table::from("tagging")),
            constraint: JoinConstraint::On(join_cond),
        };

        let clause = res.unwrap().1;
        assert_eq!(clause, join);
        assert_eq!(str, format!("{}", clause));
    }
}

use std::fmt;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use base::column::Column;
use base::condition::ConditionExpression;
use base::error::ParseSQLError;
use base::Literal;

/// ```sql
/// CASE expression
///     WHEN {value1 | condition1} THEN result1
///     ...
///     ELSE resultN
/// END
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CaseWhenExpression {
    pub condition: ConditionExpression,
    pub then_expr: ColumnOrLiteral,
    pub else_expr: Option<ColumnOrLiteral>,
}

impl CaseWhenExpression {
    pub fn parse(i: &str) -> IResult<&str, CaseWhenExpression, ParseSQLError<&str>> {
        let (input, (_, _, _, _, condition, _, _, _, column, _, else_val, _)) = tuple((
            tag_no_case("CASE"),
            multispace1,
            tag_no_case("WHEN"),
            multispace0,
            ConditionExpression::condition_expr,
            multispace0,
            tag_no_case("THEN"),
            multispace0,
            Column::without_alias,
            multispace0,
            opt(delimited(
                terminated(tag_no_case("ELSE"), multispace0),
                Literal::parse,
                multispace0,
            )),
            tag_no_case("END"),
        ))(i)?;

        let then_expr = ColumnOrLiteral::Column(column);
        let else_expr = else_val.map(ColumnOrLiteral::Literal);

        Ok((
            input,
            CaseWhenExpression {
                condition,
                then_expr,
                else_expr,
            },
        ))
    }
}

impl fmt::Display for CaseWhenExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CASE WHEN {} THEN {}", self.condition, self.then_expr)?;
        if let Some(ref expr) = self.else_expr {
            write!(f, " ELSE {}", expr)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ColumnOrLiteral {
    Column(Column),
    Literal(Literal),
}

impl fmt::Display for ColumnOrLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ColumnOrLiteral::Column(ref c) => write!(f, "{}", c)?,
            ColumnOrLiteral::Literal(ref l) => write!(f, "{}", l)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::condition::ConditionBase::{Field, Literal};
    use base::condition::ConditionExpression::{Base, ComparisonOp};
    use base::condition::ConditionTree;
    use base::Literal::Integer;
    use base::Operator::Greater;
    use base::{CaseWhenExpression, Column, ColumnOrLiteral};

    #[test]
    fn parse_case() {
        let str = "CASE WHEN age > 10 THEN col_name ELSE 22 END;";
        let res = CaseWhenExpression::parse(str);

        let exp = CaseWhenExpression {
            condition: ComparisonOp(ConditionTree {
                operator: Greater,
                left: Box::new(Base(Field(Column {
                    name: "age".to_string(),
                    alias: None,
                    table: None,
                    function: None,
                }))),
                right: Box::new(Base(Literal(Integer(10)))),
            }),
            then_expr: ColumnOrLiteral::Column(Column {
                name: "col_name".to_string(),
                alias: None,
                table: None,
                function: None,
            }),
            else_expr: Some(ColumnOrLiteral::Literal(Integer(22))),
        };

        assert!(res.is_ok());
        assert_eq!(res.unwrap().1, exp);
    }
}

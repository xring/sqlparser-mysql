use std::fmt;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::error::VerboseError;
use nom::IResult;
use nom::sequence::{delimited, terminated, tuple};

use base::column::Column;
use base::Literal;
use dms::condition::{condition_expr, ConditionExpression};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ColumnOrLiteral {
    Column(Column),
    Literal(Literal),
}

impl fmt::Display for ColumnOrLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ColumnOrLiteral::Column(ref c) => write!(f, "{}", c)?,
            ColumnOrLiteral::Literal(ref l) => write!(f, "{}", l.to_string())?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CaseWhenExpression {
    pub condition: ConditionExpression,
    pub then_expr: ColumnOrLiteral,
    pub else_expr: Option<ColumnOrLiteral>,
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

pub fn case_when_column(i: &str) -> IResult<&str, CaseWhenExpression, VerboseError<&str>> {
    let (remaining_input, (_, _, condition, _, _, _, column, _, else_val, _)) = tuple((
        tag_no_case("CASE WHEN"),
        multispace0,
        condition_expr,
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
    let else_expr = else_val.map(|v| ColumnOrLiteral::Literal(v));

    Ok((
        remaining_input,
        CaseWhenExpression {
            condition,
            then_expr,
            else_expr,
        },
    ))
}
